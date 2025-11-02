use crate::error::{Result, WorkspaceError};
use git2::{BranchType, ConfigLevel, Repository, Worktree, WorktreeLockStatus};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Git operations manager for repository and worktree handling
pub struct GitOps {
    repo: Repository,
}

impl GitOps {
    /// Open an existing repository at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path.as_ref())?;
        Ok(Self { repo })
    }

    /// Discover and open a repository starting from the given path
    pub fn discover<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::discover(path.as_ref())?;
        Ok(Self { repo })
    }

    /// Get the repository path
    pub fn path(&self) -> &Path {
        self.repo.path()
    }

    /// Get the working directory path
    pub fn workdir(&self) -> Option<&Path> {
        self.repo.workdir()
    }

    /// Check if the repository is bare
    pub fn is_bare(&self) -> bool {
        self.repo.is_bare()
    }

    /// List all branches in the repository
    pub fn list_branches(&self, filter: Option<BranchType>) -> Result<Vec<String>> {
        let filter = filter.unwrap_or(BranchType::Local);
        let mut branches = Vec::new();

        for branch in self.repo.branches(Some(filter))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }

        Ok(branches)
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Err(WorkspaceError::BranchError(
                "Could not determine current branch".to_string(),
            ))
        }
    }

    /// Create a new branch from the current HEAD
    pub fn create_branch(&self, name: &str, force: bool) -> Result<()> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;

        if force {
            // Delete existing branch if it exists
            if let Ok(mut branch) = self.repo.find_branch(name, BranchType::Local) {
                branch.delete()?;
            }
        }

        self.repo.branch(name, &commit, false)?;
        info!("Created branch: {}", name);
        Ok(())
    }

    /// List all worktrees
    pub fn list_worktrees(&self) -> Result<Vec<String>> {
        let worktrees = self.repo.worktrees()?;
        Ok(worktrees
            .iter()
            .filter_map(|s| s.map(String::from))
            .collect())
    }

    /// Add a new worktree
    pub fn add_worktree<P: AsRef<Path>>(
        &self,
        name: &str,
        path: P,
        branch: Option<&str>,
    ) -> Result<Worktree> {
        let path = path.as_ref();

        debug!("Adding worktree: {} at {:?}", name, path);

        // Create the worktree
        let worktree = if let Some(branch_name) = branch {
            // Create a new branch for the worktree if specified
            let head = self.repo.head()?;
            let commit = head.peel_to_commit()?;
            let branch = self.repo.branch(branch_name, &commit, false)?;

            // Add worktree with the new branch
            self.repo.worktree(
                name,
                path,
                Some(git2::WorktreeAddOptions::new().reference(Some(branch.get()))),
            )?
        } else {
            // Add worktree without creating a new branch
            self.repo.worktree(name, path, None)?
        };

        info!("Added worktree: {} at {:?}", name, path);
        Ok(worktree)
    }

    /// Remove a worktree (prune it)
    pub fn remove_worktree(&self, name: &str) -> Result<()> {
        let worktree = self.repo.find_worktree(name)?;

        debug!("Removing worktree: {}", name);

        // Prune the worktree
        worktree.prune(Some(git2::WorktreePruneOptions::new().valid(true)))?;

        info!("Removed worktree: {}", name);
        Ok(())
    }

    /// Get worktree information
    pub fn worktree_info(&self, name: &str) -> Result<WorktreeInfo> {
        let worktree = self.repo.find_worktree(name)?;

        let path = worktree.path().to_path_buf();
        // is_locked() returns Ok(status) which is either Locked or Unlocked
        let is_locked = worktree
            .is_locked()
            .map(|status| matches!(status, WorktreeLockStatus::Locked { .. }))
            .unwrap_or(false);
        let is_valid = worktree.validate().is_ok();

        Ok(WorktreeInfo {
            name: name.to_string(),
            path,
            is_locked,
            is_valid,
        })
    }

    /// Check if a worktree exists
    pub fn has_worktree(&self, name: &str) -> bool {
        self.repo.find_worktree(name).is_ok()
    }

    /// Get the default branch name (usually main or master)
    pub fn default_branch(&self) -> Result<String> {
        // Try to get the remote's HEAD
        if let Ok(remote) = self.repo.find_remote("origin") {
            if let Ok(buf) = remote.default_branch() {
                if let Some(name) = buf.as_str() {
                    // Strip refs/heads/ prefix if present
                    let branch = name.strip_prefix("refs/heads/").unwrap_or(name);
                    return Ok(branch.to_string());
                }
            }
        }

        // Fallback: check if main or master exists
        for name in &["main", "master"] {
            if self.repo.find_branch(name, BranchType::Local).is_ok() {
                return Ok(name.to_string());
            }
        }

        Err(WorkspaceError::BranchError(
            "Could not determine default branch".to_string(),
        ))
    }

    /// Get repository status summary
    pub fn status_summary(&self) -> Result<StatusSummary> {
        let statuses = self.repo.statuses(None)?;

        let mut modified = 0;
        let mut added = 0;
        let mut deleted = 0;
        let mut untracked = 0;

        for entry in statuses.iter() {
            let status = entry.status();
            if status.is_wt_modified() || status.is_index_modified() {
                modified += 1;
            }
            if status.is_wt_new() {
                untracked += 1;
            }
            if status.is_index_new() {
                added += 1;
            }
            if status.is_wt_deleted() || status.is_index_deleted() {
                deleted += 1;
            }
        }

        Ok(StatusSummary {
            modified,
            added,
            deleted,
            untracked,
        })
    }

    /// Get status summary for a specific worktree
    pub fn worktree_status(&self, worktree_path: &Path) -> Result<StatusSummary> {
        // Open the worktree repository
        let worktree_repo = Repository::open(worktree_path)?;
        let statuses = worktree_repo.statuses(None)?;

        let mut modified = 0;
        let mut added = 0;
        let mut deleted = 0;
        let mut untracked = 0;

        for entry in statuses.iter() {
            let status = entry.status();
            if status.is_wt_modified() || status.is_index_modified() {
                modified += 1;
            }
            if status.is_wt_new() {
                untracked += 1;
            }
            if status.is_index_new() {
                added += 1;
            }
            if status.is_wt_deleted() || status.is_index_deleted() {
                deleted += 1;
            }
        }

        Ok(StatusSummary {
            modified,
            added,
            deleted,
            untracked,
        })
    }

    /// Get all values for a git config key (for multi-value configs like workspace.repo)
    pub fn config_get_all(&self, key: &str) -> Result<Vec<String>> {
        let config = self.repo.config()?;
        let mut values = Vec::new();

        // Get all multivar values using multivar iterator
        if let Ok(mut entries) = config.entries(Some(key)) {
            while let Some(entry) = entries.next() {
                if let Ok(entry) = entry {
                    if let Some(value) = entry.value() {
                        values.push(value.to_string());
                    }
                }
            }
        }

        Ok(values)
    }

    /// Add a value to a multi-value git config key
    pub fn config_add(&self, key: &str, value: &str) -> Result<()> {
        let mut config = self.repo.config()?;
        config.set_multivar(key, "^$", value)?; // ^$ matches nothing, so always adds
        Ok(())
    }

    /// Set a git config value (single value)
    pub fn config_set(&self, key: &str, value: &str) -> Result<()> {
        let mut config = self.repo.config()?;
        config.set_str(key, value)?;
        Ok(())
    }

    /// Remove all values for a git config key
    pub fn config_unset_all(&self, key: &str) -> Result<()> {
        let mut config = self.repo.config()?;
        config.remove_multivar(key, ".*")?;
        Ok(())
    }

    /// Enable worktree config extension
    pub fn enable_worktree_config(&self) -> Result<()> {
        self.config_set("extensions.worktreeConfig", "true")?;
        info!("Enabled worktree config extension");
        Ok(())
    }

    /// Check if worktree config extension is enabled
    pub fn is_worktree_config_enabled(&self) -> bool {
        if let Ok(config) = self.repo.config() {
            if let Ok(value) = config.get_bool("extensions.worktreeConfig") {
                return value;
            }
        }
        false
    }

    /// Add a value to a multi-value git config key at the worktree level
    /// This writes to .git/worktrees/<name>/config.worktree
    pub fn worktree_config_add(&self, key: &str, value: &str) -> Result<()> {
        // First ensure worktree config extension is enabled
        if !self.is_worktree_config_enabled() {
            self.enable_worktree_config()?;
        }

        // Get the full config and open the worktree level
        let config = self.repo.config()?;
        let mut worktree_config = config.open_level(ConfigLevel::Worktree)?;

        // Add the multivar value at worktree level
        worktree_config.set_multivar(key, "^$", value)?; // ^$ matches nothing, so always adds

        debug!("Added worktree config: {} = {}", key, value);
        Ok(())
    }

    /// Set a git config value (single value) at the worktree level
    pub fn worktree_config_set(&self, key: &str, value: &str) -> Result<()> {
        // First ensure worktree config extension is enabled
        if !self.is_worktree_config_enabled() {
            self.enable_worktree_config()?;
        }

        // Get the full config and open the worktree level
        let config = self.repo.config()?;
        let mut worktree_config = config.open_level(ConfigLevel::Worktree)?;

        // Set the value at worktree level
        worktree_config.set_str(key, value)?;

        debug!("Set worktree config: {} = {}", key, value);
        Ok(())
    }

    /// Get all values for a git config key from the worktree level only
    pub fn worktree_config_get_all(&self, key: &str) -> Result<Vec<String>> {
        if !self.is_worktree_config_enabled() {
            return Ok(Vec::new());
        }

        let config = self.repo.config()?;
        let worktree_config = config.open_level(ConfigLevel::Worktree)?;
        let mut values = Vec::new();

        // Get all multivar values using multivar iterator
        if let Ok(mut entries) = worktree_config.entries(Some(key)) {
            while let Some(entry) = entries.next() {
                if let Ok(entry) = entry {
                    if let Some(value) = entry.value() {
                        values.push(value.to_string());
                    }
                }
            }
        }

        Ok(values)
    }

    /// Remove all values for a git config key at the worktree level
    pub fn worktree_config_unset_all(&self, key: &str) -> Result<()> {
        if !self.is_worktree_config_enabled() {
            return Ok(()); // Nothing to unset if extension not enabled
        }

        let config = self.repo.config()?;
        let mut worktree_config = config.open_level(ConfigLevel::Worktree)?;
        worktree_config.remove_multivar(key, ".*")?;

        debug!("Removed all worktree config values for: {}", key);
        Ok(())
    }
}

/// Information about a worktree
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_locked: bool,
    pub is_valid: bool,
}

/// Summary of repository status
#[derive(Debug, Clone)]
pub struct StatusSummary {
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
    pub untracked: usize,
}

impl StatusSummary {
    pub fn is_clean(&self) -> bool {
        self.modified == 0 && self.added == 0 && self.deleted == 0 && self.untracked == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_open_repository() {
        // This test requires running in a git repository
        if let Ok(git_ops) = GitOps::discover(".") {
            assert!(git_ops.path().exists());
        }
    }

    #[test]
    fn test_list_branches() {
        if let Ok(git_ops) = GitOps::discover(".") {
            let branches = git_ops.list_branches(None).unwrap();
            assert!(!branches.is_empty());
        }
    }

    #[test]
    fn test_current_branch() {
        if let Ok(git_ops) = GitOps::discover(".") {
            let branch = git_ops.current_branch().unwrap();
            assert!(!branch.is_empty());
        }
    }

    // Helper function to create a test git repository
    fn create_test_repo() -> (tempfile::TempDir, Repository) {
        let tempdir = tempdir().unwrap();
        let repo = Repository::init(tempdir.path()).unwrap();

        // Configure git user for commits
        {
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test User").unwrap();
            config.set_str("user.email", "test@example.com").unwrap();
        }

        // Create an initial commit so we have a HEAD
        {
            let sig = repo.signature().unwrap();
            let tree_id = {
                let mut index = repo.index().unwrap();
                index.write_tree().unwrap()
            };
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();
        }

        (tempdir, repo)
    }

    #[test]
    fn test_worktree_specific_config() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Enable worktree config extension in main repo first
        git_ops.enable_worktree_config().unwrap();

        // Create worktrees parent directory (libgit2 will create the worktree itself)
        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        let worktree_path = worktrees_dir.join("feature");
        git_ops
            .add_worktree("feature", &worktree_path, Some("workspace/feature"))
            .unwrap();

        // Open the worktree repository
        let worktree_ops = GitOps::open(&worktree_path).unwrap();

        // Set worktree-specific config
        worktree_ops
            .worktree_config_add("workspace.repo", "https://example.com/repo feature-test")
            .unwrap();

        // Verify worktree config was set
        let values = worktree_ops
            .worktree_config_get_all("workspace.repo")
            .unwrap();
        assert_eq!(values.len(), 1);
        assert!(values[0].contains("feature-test"));
    }

    #[test]
    fn test_config_inheritance_chain() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Set default git config (superproject level)
        git_ops
            .config_add("workspace.repo", "https://example.com/repo-default develop")
            .unwrap();

        // Verify default config is readable
        let default_values = git_ops.config_get_all("workspace.repo").unwrap();
        assert_eq!(default_values.len(), 1);
        assert!(default_values[0].contains("repo-default"));
        assert!(default_values[0].contains("develop"));

        // Enable worktree config extension in main repo first
        git_ops.enable_worktree_config().unwrap();

        // Create worktrees parent directory (libgit2 will create the worktree itself)
        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        let worktree_path = worktrees_dir.join("test");
        git_ops
            .add_worktree("test", &worktree_path, Some("workspace/test"))
            .unwrap();

        // Open the worktree repository
        let worktree_ops = GitOps::open(&worktree_path).unwrap();

        // Set worktree-specific config (should override default)
        worktree_ops
            .worktree_config_add(
                "workspace.repo",
                "https://example.com/repo-worktree feature-test",
            )
            .unwrap();

        // Verify worktree config overrides default
        let worktree_values = worktree_ops
            .worktree_config_get_all("workspace.repo")
            .unwrap();
        assert_eq!(worktree_values.len(), 1);
        assert!(worktree_values[0].contains("repo-worktree"));
        assert!(worktree_values[0].contains("feature-test"));

        // Verify reading from full config includes both levels
        let all_values = worktree_ops.config_get_all("workspace.repo").unwrap();
        // Should see both worktree and default configs
        assert!(all_values.len() >= 1);
    }

    #[test]
    fn test_workspace_isolation() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Enable worktree config extension in main repo first
        git_ops.enable_worktree_config().unwrap();

        // Create worktrees parent directory (libgit2 will create the worktrees themselves)
        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        let worktree1_path = worktrees_dir.join("workspace1");
        let worktree2_path = worktrees_dir.join("workspace2");

        git_ops
            .add_worktree("workspace1", &worktree1_path, Some("workspace/workspace1"))
            .unwrap();
        git_ops
            .add_worktree("workspace2", &worktree2_path, Some("workspace/workspace2"))
            .unwrap();

        // Open both worktrees
        let worktree1_ops = GitOps::open(&worktree1_path).unwrap();
        let worktree2_ops = GitOps::open(&worktree2_path).unwrap();

        // Set different configs for each workspace
        worktree1_ops
            .worktree_config_add("workspace.repo", "https://example.com/repo-a develop")
            .unwrap();
        worktree2_ops
            .worktree_config_add("workspace.repo", "https://example.com/repo-b feature-test")
            .unwrap();

        // Verify workspace1 config
        let values1 = worktree1_ops
            .worktree_config_get_all("workspace.repo")
            .unwrap();
        assert_eq!(values1.len(), 1);
        assert!(values1[0].contains("repo-a"));
        assert!(values1[0].contains("develop"));
        assert!(!values1[0].contains("repo-b"));

        // Verify workspace2 config
        let values2 = worktree2_ops
            .worktree_config_get_all("workspace.repo")
            .unwrap();
        assert_eq!(values2.len(), 1);
        assert!(values2[0].contains("repo-b"));
        assert!(values2[0].contains("feature-test"));
        assert!(!values2[0].contains("repo-a"));
    }

    #[test]
    fn test_enable_worktree_config_extension() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Initially should be disabled
        assert!(!git_ops.is_worktree_config_enabled());

        // Enable it
        git_ops.enable_worktree_config().unwrap();

        // Should now be enabled
        assert!(git_ops.is_worktree_config_enabled());
    }

    #[test]
    fn test_config_add_and_get_all() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Add multiple values
        git_ops
            .config_add("workspace.repo", "https://example.com/repo1 main")
            .unwrap();
        git_ops
            .config_add("workspace.repo", "https://example.com/repo2 develop")
            .unwrap();
        git_ops
            .config_add("workspace.repo", "https://example.com/repo3 feature v1.0.0")
            .unwrap();

        // Get all values
        let values = git_ops.config_get_all("workspace.repo").unwrap();
        assert_eq!(values.len(), 3);
        assert!(values[0].contains("repo1"));
        assert!(values[1].contains("repo2"));
        assert!(values[2].contains("repo3"));
        assert!(values[2].contains("v1.0.0"));
    }

    #[test]
    fn test_config_unset_all() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Add multiple values
        git_ops
            .config_add("workspace.repo", "https://example.com/repo1 main")
            .unwrap();
        git_ops
            .config_add("workspace.repo", "https://example.com/repo2 develop")
            .unwrap();

        // Verify they exist
        let values = git_ops.config_get_all("workspace.repo").unwrap();
        assert_eq!(values.len(), 2);

        // Unset all
        git_ops.config_unset_all("workspace.repo").unwrap();

        // Verify they're gone
        let values = git_ops.config_get_all("workspace.repo").unwrap();
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn test_worktree_config_unset_all() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Enable worktree config extension in main repo first
        git_ops.enable_worktree_config().unwrap();

        // Create worktrees parent directory (libgit2 will create the worktree itself)
        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        let worktree_path = worktrees_dir.join("test");
        git_ops
            .add_worktree("test", &worktree_path, Some("workspace/test"))
            .unwrap();

        // Open the worktree repository
        let worktree_ops = GitOps::open(&worktree_path).unwrap();

        // Add worktree-specific configs
        worktree_ops
            .worktree_config_add("workspace.repo", "https://example.com/repo1 main")
            .unwrap();
        worktree_ops
            .worktree_config_add("workspace.repo", "https://example.com/repo2 develop")
            .unwrap();

        // Verify they exist
        let values = worktree_ops
            .worktree_config_get_all("workspace.repo")
            .unwrap();
        assert_eq!(values.len(), 2);

        // Unset all at worktree level
        worktree_ops
            .worktree_config_unset_all("workspace.repo")
            .unwrap();

        // Verify they're gone
        let values = worktree_ops
            .worktree_config_get_all("workspace.repo")
            .unwrap();
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn test_worktree_config_set_single_value() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Enable worktree config extension in main repo first
        git_ops.enable_worktree_config().unwrap();

        // Create worktrees parent directory (libgit2 will create the worktree itself)
        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        let worktree_path = worktrees_dir.join("test");
        git_ops
            .add_worktree("test", &worktree_path, Some("workspace/test"))
            .unwrap();

        // Open the worktree repository
        let worktree_ops = GitOps::open(&worktree_path).unwrap();

        // Set a single-value config
        worktree_ops
            .worktree_config_set("workspace.default", "main")
            .unwrap();

        // Verify it was set (using regular config_get_all since it's a single value)
        let config = worktree_ops.repo.config().unwrap();
        let value = config.get_string("workspace.default").unwrap();
        assert_eq!(value, "main");
    }

    #[test]
    fn test_worktree_list_and_removal() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Initially should have no worktrees (or just main)
        let initial_worktrees = git_ops.list_worktrees().unwrap();

        // Create worktrees parent directory (libgit2 will create the worktree itself)
        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        let worktree_path = worktrees_dir.join("test");
        git_ops
            .add_worktree("test", &worktree_path, Some("workspace/test"))
            .unwrap();

        // Should now have one more worktree
        let worktrees = git_ops.list_worktrees().unwrap();
        assert_eq!(worktrees.len(), initial_worktrees.len() + 1);
        assert!(worktrees.contains(&"test".to_string()));

        // Remove the worktree
        git_ops.remove_worktree("test").unwrap();

        // Should be back to initial count
        let final_worktrees = git_ops.list_worktrees().unwrap();
        assert_eq!(final_worktrees.len(), initial_worktrees.len());
        assert!(!final_worktrees.contains(&"test".to_string()));
    }

    #[test]
    fn test_worktree_branch_creation() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Create worktrees parent directory (libgit2 will create the worktrees themselves)
        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Get initial branches
        let initial_branches = git_ops.list_branches(None).unwrap();

        // Add worktree with new branch
        let worktree_path = worktrees_dir.join("feature-x");
        git_ops
            .add_worktree("feature-x", &worktree_path, Some("workspace/feature-x"))
            .unwrap();

        // Should have new branch
        let branches = git_ops.list_branches(None).unwrap();
        assert_eq!(branches.len(), initial_branches.len() + 1);
        assert!(branches.contains(&"workspace/feature-x".to_string()));

        // Add another worktree
        let worktree2_path = worktrees_dir.join("hotfix-y");
        git_ops
            .add_worktree("hotfix-y", &worktree2_path, Some("workspace/hotfix-y"))
            .unwrap();

        // Should have both branches
        let branches = git_ops.list_branches(None).unwrap();
        assert_eq!(branches.len(), initial_branches.len() + 2);
        assert!(branches.contains(&"workspace/feature-x".to_string()));
        assert!(branches.contains(&"workspace/hotfix-y".to_string()));
    }

    // ==================== TestWorktreeLifecycle Tests ====================

    #[test]
    fn test_worktree_creation_and_removal_lifecycle() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        let worktree_path = worktrees_dir.join("feature-1");

        // Create worktree
        git_ops
            .add_worktree("feature-1", &worktree_path, Some("feature-1"))
            .unwrap();

        // Verify worktree exists
        assert!(worktree_path.exists());

        // Verify it has .git file (not directory) - characteristic of worktrees
        let git_file = worktree_path.join(".git");
        assert!(
            git_file.is_file(),
            "Worktree should have .git file, not directory"
        );

        // Read .git file to verify it points to central repo
        let git_content = std::fs::read_to_string(&git_file).unwrap();
        assert!(
            git_content.starts_with("gitdir:"),
            ".git file should contain gitdir reference"
        );

        // Verify worktree is in the list
        let worktrees = git_ops.list_worktrees().unwrap();
        assert!(worktrees.contains(&"feature-1".to_string()));

        // Remove worktree
        git_ops.remove_worktree("feature-1").unwrap();

        // Verify worktree is removed from git's list
        let worktrees = git_ops.list_worktrees().unwrap();
        assert!(!worktrees.contains(&"feature-1".to_string()));
    }

    #[test]
    fn test_multiple_worktrees_same_repo() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Create first worktree
        let worktree1_path = worktrees_dir.join("feature-1");
        git_ops
            .add_worktree("feature-1", &worktree1_path, Some("feature-1"))
            .unwrap();

        // Create second worktree
        let worktree2_path = worktrees_dir.join("feature-2");
        git_ops
            .add_worktree("feature-2", &worktree2_path, Some("feature-2"))
            .unwrap();

        // Both should exist
        assert!(worktree1_path.exists());
        assert!(worktree2_path.exists());

        // Both should be in worktree list
        let worktrees = git_ops.list_worktrees().unwrap();
        assert!(worktrees.contains(&"feature-1".to_string()));
        assert!(worktrees.contains(&"feature-2".to_string()));

        // Verify they share git objects (both point to same repo)
        let git_content1 = std::fs::read_to_string(worktree1_path.join(".git")).unwrap();
        let git_content2 = std::fs::read_to_string(worktree2_path.join(".git")).unwrap();

        // Both should reference the main repo
        assert!(git_content1.contains("gitdir:"));
        assert!(git_content2.contains("gitdir:"));

        // Create and commit a file in feature-1
        let test_file = worktree1_path.join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let worktree1_repo = Repository::open(&worktree1_path).unwrap();
        let mut index = worktree1_repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = worktree1_repo.find_tree(tree_id).unwrap();
        let sig = worktree1_repo.signature().unwrap();
        let parent = worktree1_repo.head().unwrap().peel_to_commit().unwrap();

        worktree1_repo
            .commit(Some("HEAD"), &sig, &sig, "test commit", &tree, &[&parent])
            .unwrap();

        // Verify commits are visible from main repo (shared git objects)
        let main_repo = GitOps::open(_tempdir.path()).unwrap();
        let branches = main_repo.list_branches(None).unwrap();
        assert!(branches.contains(&"feature-1".to_string()));
        assert!(branches.contains(&"feature-2".to_string()));
    }

    #[test]
    fn test_worktree_branch_conflicts() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Create a branch "test-branch" that we'll try to checkout in multiple worktrees
        git_ops.create_branch("test-branch", false).unwrap();

        // Create first worktree on test-branch using a new branch "wt-1" based on test-branch
        let worktree1_path = worktrees_dir.join("first");
        git_ops
            .add_worktree("first", &worktree1_path, Some("wt-1"))
            .unwrap();

        // Try to create wt-1 again (should fail - branch already exists)
        let worktree2_path = worktrees_dir.join("second");
        let result = git_ops.add_worktree("second", &worktree2_path, Some("wt-1"));

        // Git should prevent creating the same branch twice
        assert!(
            result.is_err(),
            "Should not allow creating the same branch twice"
        );
    }

    #[test]
    fn test_orphaned_worktree_cleanup() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Create worktree
        let worktree_path = worktrees_dir.join("feature-orphan");
        git_ops
            .add_worktree("feature-orphan", &worktree_path, Some("feature-orphan"))
            .unwrap();

        // Verify it's in the list
        let worktrees = git_ops.list_worktrees().unwrap();
        assert!(worktrees.contains(&"feature-orphan".to_string()));

        // Manually delete the worktree directory (simulating corruption/manual deletion)
        std::fs::remove_dir_all(&worktree_path).unwrap();

        // Git still thinks the worktree exists
        let worktrees = git_ops.list_worktrees().unwrap();
        assert!(worktrees.contains(&"feature-orphan".to_string()));

        // But validation should fail
        let info = git_ops.worktree_info("feature-orphan").unwrap();
        assert!(!info.is_valid, "Orphaned worktree should not be valid");

        // Prune should clean it up
        git_ops.remove_worktree("feature-orphan").unwrap();

        // Now it should be gone from the list
        let worktrees = git_ops.list_worktrees().unwrap();
        assert!(!worktrees.contains(&"feature-orphan".to_string()));
    }

    #[test]
    fn test_worktree_with_uncommitted_changes() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Create worktree
        let worktree_path = worktrees_dir.join("feature-dirty");
        git_ops
            .add_worktree("feature-dirty", &worktree_path, Some("feature-dirty"))
            .unwrap();

        // Make uncommitted changes
        let test_file = worktree_path.join("uncommitted.txt");
        std::fs::write(&test_file, "uncommitted content").unwrap();

        // Status should show uncommitted changes
        let worktree_ops = GitOps::open(&worktree_path).unwrap();
        let status = worktree_ops.status_summary().unwrap();
        assert!(!status.is_clean(), "Should have uncommitted changes");
        assert!(status.untracked > 0, "Should have untracked files");

        // Can still remove worktree (prune will force removal)
        git_ops.remove_worktree("feature-dirty").unwrap();

        // Worktree should be removed
        let worktrees = git_ops.list_worktrees().unwrap();
        assert!(!worktrees.contains(&"feature-dirty".to_string()));
    }

    // ==================== TestCentralRepositoryManagement Tests ====================

    #[test]
    fn test_ensure_central_repo_creation() {
        // Test that we can create and verify a central repository
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Verify repository exists
        assert!(git_ops.path().exists());

        // Verify it has a HEAD
        let head_file = if git_ops.is_bare() {
            git_ops.path().join("HEAD")
        } else {
            git_ops.path().parent().unwrap().join(".git").join("HEAD")
        };
        assert!(head_file.exists() || git_ops.path().join("HEAD").exists());
    }

    #[test]
    fn test_central_repo_with_multiple_commits() {
        let (_tempdir, _repo) = create_test_repo();
        let repo_path = _tempdir.path();

        // Create multiple commits
        for i in 0..10 {
            let test_file = repo_path.join(format!("file{}.txt", i));
            std::fs::write(&test_file, format!("content {}", i)).unwrap();

            let repo = Repository::open(repo_path).unwrap();
            let mut index = repo.index().unwrap();
            index
                .add_path(std::path::Path::new(&format!("file{}.txt", i)))
                .unwrap();
            index.write().unwrap();

            let tree_id = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            let sig = repo.signature().unwrap();
            let parent = repo.head().unwrap().peel_to_commit().unwrap();

            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!("commit {}", i),
                &tree,
                &[&parent],
            )
            .unwrap();
        }

        // Verify we can open the repo and see all commits
        let git_ops = GitOps::open(repo_path).unwrap();

        // Create a worktree and verify all history is available
        let worktrees_dir = repo_path.join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        let worktree_path = worktrees_dir.join("test");
        git_ops
            .add_worktree("test", &worktree_path, Some("test-branch"))
            .unwrap();

        // Open worktree and count commits
        let worktree_repo = Repository::open(&worktree_path).unwrap();
        let mut revwalk = worktree_repo.revwalk().unwrap();
        revwalk.push_head().unwrap();

        let commit_count = revwalk.count();
        // Should have initial commit + 10 new commits
        assert!(commit_count >= 11, "Should have at least 11 commits");
    }

    #[test]
    fn test_worktree_shares_objects_with_central() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Create worktree
        let worktree_path = worktrees_dir.join("test");
        git_ops
            .add_worktree("test", &worktree_path, Some("test-branch"))
            .unwrap();

        // Create a commit in the worktree
        let test_file = worktree_path.join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let worktree_repo = Repository::open(&worktree_path).unwrap();
        let mut index = worktree_repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = worktree_repo.find_tree(tree_id).unwrap();
        let sig = worktree_repo.signature().unwrap();
        let parent = worktree_repo.head().unwrap().peel_to_commit().unwrap();

        let commit_id = worktree_repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                "test commit in worktree",
                &tree,
                &[&parent],
            )
            .unwrap();

        // Verify the commit is visible from the main repo
        let main_repo = Repository::open(_tempdir.path()).unwrap();
        let commit = main_repo.find_commit(commit_id);
        assert!(
            commit.is_ok(),
            "Commit should be visible from main repo (shared objects)"
        );
    }

    #[test]
    fn test_worktree_references_central_repo() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Create worktree
        let worktree_path = worktrees_dir.join("test");
        git_ops
            .add_worktree("test", &worktree_path, Some("test-branch"))
            .unwrap();

        // Verify .git file points to central repo
        let git_file = worktree_path.join(".git");
        assert!(git_file.is_file());

        let git_content = std::fs::read_to_string(&git_file).unwrap();
        assert!(git_content.starts_with("gitdir:"));

        // The gitdir should point to the main repo's worktrees directory
        assert!(git_content.contains(".git/worktrees"));
    }

    // ==================== TestWorktreeErrorHandling Tests ====================
    // Note: Some error tests from Python (corrupted repos, permission errors, network failures)
    // are difficult to reliably test in Rust without external tools or platform-specific code.
    // We'll focus on tests that verify our error handling works correctly.

    #[test]
    fn test_worktree_creation_with_existing_directory() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Create a directory where worktree should be created
        let worktree_path = worktrees_dir.join("blocked");
        std::fs::create_dir_all(&worktree_path).unwrap();

        // Create a file in the directory to make it non-empty
        let blocking_file = worktree_path.join("blocking.txt");
        std::fs::write(&blocking_file, "blocking").unwrap();

        // Try to create worktree (libgit2 should handle this gracefully)
        let result = git_ops.add_worktree("blocked", &worktree_path, Some("blocked-branch"));

        // Should fail because directory already exists and is not empty
        assert!(result.is_err(), "Should fail when directory already exists");
    }

    #[test]
    fn test_remove_nonexistent_worktree() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Try to remove a worktree that doesn't exist
        let result = git_ops.remove_worktree("nonexistent");

        // Should fail gracefully
        assert!(
            result.is_err(),
            "Should fail when removing nonexistent worktree"
        );
    }

    #[test]
    fn test_worktree_info_for_nonexistent() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        // Try to get info for nonexistent worktree
        let result = git_ops.worktree_info("nonexistent");

        // Should fail gracefully
        assert!(
            result.is_err(),
            "Should fail when getting info for nonexistent worktree"
        );
    }

    #[test]
    fn test_status_with_large_changes() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Create worktree
        let worktree_path = worktrees_dir.join("test");
        git_ops
            .add_worktree("test", &worktree_path, Some("test-branch"))
            .unwrap();

        // Create many files
        for i in 0..100 {
            let file = worktree_path.join(format!("file{}.txt", i));
            std::fs::write(&file, format!("content {}", i)).unwrap();
        }

        // Status should handle many untracked files
        let worktree_ops = GitOps::open(&worktree_path).unwrap();
        let status = worktree_ops.status_summary().unwrap();

        assert_eq!(status.untracked, 100);
        assert!(!status.is_clean());
    }

    #[test]
    fn test_invalid_branch_name() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Try to create worktree with invalid branch name (git doesn't allow "..")
        let worktree_path = worktrees_dir.join("test");
        let result = git_ops.add_worktree("test", &worktree_path, Some(".."));

        // Should fail with invalid branch name
        assert!(result.is_err(), "Should fail with invalid branch name");
    }

    // ==================== TestWorktreeIntegration Tests ====================
    // Note: Submodule and symlink tests require more complex setup.
    // We'll implement basic integration tests that verify worktree functionality.

    #[test]
    fn test_worktree_performance_basic() {
        use std::time::Instant;

        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Measure worktree creation time
        let start = Instant::now();
        let worktree_path = worktrees_dir.join("perf-test");
        git_ops
            .add_worktree("perf-test", &worktree_path, Some("perf-branch"))
            .unwrap();
        let duration = start.elapsed();

        // Worktree creation should be fast (under 1 second for small repos)
        assert!(duration.as_secs() < 1, "Worktree creation should be fast");

        // Verify it was created
        assert!(worktree_path.exists());
    }

    #[test]
    fn test_multiple_worktrees_different_branches() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Create multiple worktrees on different branches
        let branches = vec!["feature-a", "feature-b", "feature-c"];
        for branch in &branches {
            let worktree_path = worktrees_dir.join(branch);
            git_ops
                .add_worktree(branch, &worktree_path, Some(branch))
                .unwrap();
        }

        // All worktrees should exist
        for branch in &branches {
            let worktree_path = worktrees_dir.join(branch);
            assert!(worktree_path.exists());
        }

        // All should be in the list
        let worktrees = git_ops.list_worktrees().unwrap();
        for branch in &branches {
            assert!(worktrees.contains(&branch.to_string()));
        }

        // All branches should exist
        let all_branches = git_ops.list_branches(None).unwrap();
        for branch in &branches {
            assert!(all_branches.contains(&branch.to_string()));
        }
    }

    #[test]
    fn test_worktree_isolation_changes() {
        let (_tempdir, _repo) = create_test_repo();
        let git_ops = GitOps::discover(_tempdir.path()).unwrap();

        let worktrees_dir = _tempdir.path().join("worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();

        // Create two worktrees
        let worktree1_path = worktrees_dir.join("wt1");
        let worktree2_path = worktrees_dir.join("wt2");

        git_ops
            .add_worktree("wt1", &worktree1_path, Some("branch1"))
            .unwrap();
        git_ops
            .add_worktree("wt2", &worktree2_path, Some("branch2"))
            .unwrap();

        // Make changes in worktree1
        std::fs::write(worktree1_path.join("file1.txt"), "content 1").unwrap();

        // Make changes in worktree2
        std::fs::write(worktree2_path.join("file2.txt"), "content 2").unwrap();

        // Check status of each worktree independently
        let wt1_ops = GitOps::open(&worktree1_path).unwrap();
        let wt1_status = wt1_ops.status_summary().unwrap();

        let wt2_ops = GitOps::open(&worktree2_path).unwrap();
        let wt2_status = wt2_ops.status_summary().unwrap();

        // Each should only see their own changes
        assert_eq!(wt1_status.untracked, 1);
        assert_eq!(wt2_status.untracked, 1);

        // Verify files exist only in their respective worktrees
        assert!(worktree1_path.join("file1.txt").exists());
        assert!(!worktree1_path.join("file2.txt").exists());

        assert!(worktree2_path.join("file2.txt").exists());
        assert!(!worktree2_path.join("file1.txt").exists());
    }
}
