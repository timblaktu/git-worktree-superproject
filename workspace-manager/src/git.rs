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
}
