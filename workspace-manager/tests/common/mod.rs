//! Common test utilities and fixtures for integration tests

use rstest::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{tempdir, TempDir};
use workspace_manager::workspace::{WorkspaceConfig, WorkspaceConfigBuilder};

/// Test workspace fixture providing isolated test environment
///
/// Automatically cleans up via Drop when the fixture goes out of scope.
pub struct TestWorkspace {
    temp_dir: TempDir,
}

impl TestWorkspace {
    /// Create a new test workspace
    pub fn new() -> Self {
        Self {
            temp_dir: tempdir().expect("Failed to create temp directory"),
        }
    }

    /// Get the path to the test workspace
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Get the worktrees directory path
    pub fn worktrees_path(&self) -> PathBuf {
        self.path().join("worktrees")
    }
}

// Automatic cleanup via Drop trait - no explicit teardown needed
impl Drop for TestWorkspace {
    fn drop(&mut self) {
        // tempdir handles cleanup automatically
    }
}

/// Test git repositories fixture for creating real git repos
///
/// Provides helper methods to create bare git repositories that can be cloned
/// in integration tests.
pub struct TestGitRepos {
    temp_dir: TempDir,
    pub repos: Vec<TestRepo>,
}

impl TestGitRepos {
    /// Create a new test git repos fixture
    pub fn new() -> Self {
        Self {
            temp_dir: tempdir().expect("Failed to create temp directory"),
            repos: Vec::new(),
        }
    }

    /// Create a new git repository for testing
    ///
    /// Creates a bare repository with an initial commit on the main branch.
    pub fn create_repo(&mut self, name: &str) -> &TestRepo {
        let repo_path = self.temp_dir.path().join(name);

        // Initialize bare repository
        Command::new("git")
            .args(["init", "--bare"])
            .arg(&repo_path)
            .output()
            .expect("Failed to init bare repo");

        // Create a temporary clone to add initial commit
        let temp_clone = self.temp_dir.path().join(format!("{}-clone", name));
        Command::new("git")
            .args(["clone"])
            .arg(&repo_path)
            .arg(&temp_clone)
            .output()
            .expect("Failed to clone bare repo");

        // Create initial commit
        Command::new("git")
            .args(["commit", "--allow-empty", "-m", "Initial commit"])
            .current_dir(&temp_clone)
            .output()
            .expect("Failed to create initial commit");

        // Push to bare repo
        Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&temp_clone)
            .output()
            .expect("Failed to push to bare repo");

        // Clean up temp clone
        std::fs::remove_dir_all(&temp_clone).expect("Failed to remove temp clone");

        let url = format!("file://{}", repo_path.display());
        let repo = TestRepo {
            name: name.to_string(),
            path: repo_path,
            url,
        };

        self.repos.push(repo);
        self.repos.last().unwrap()
    }

    /// Create a repository with a specific tag
    pub fn create_repo_with_tag(&mut self, name: &str, tag: &str) -> &TestRepo {
        // Create the repo first
        let _repo = self.create_repo(name);

        // Get repo path and temp_dir path separately to avoid borrow conflicts
        let repo_path = self.repos.last().unwrap().path.clone();
        let temp_dir_path = self.temp_dir.path().to_path_buf();

        // Create a tag on the initial commit
        let temp_clone = temp_dir_path.join(format!("{}-clone", name));
        Command::new("git")
            .args(["clone"])
            .arg(&repo_path)
            .arg(&temp_clone)
            .output()
            .expect("Failed to clone for tagging");

        Command::new("git")
            .args(["tag", tag])
            .current_dir(&temp_clone)
            .output()
            .expect("Failed to create tag");

        Command::new("git")
            .args(["push", "origin", tag])
            .current_dir(&temp_clone)
            .output()
            .expect("Failed to push tag");

        std::fs::remove_dir_all(&temp_clone).expect("Failed to remove temp clone");

        self.repos.last().unwrap()
    }

    /// Add a commit to an existing repository
    pub fn add_commit(&self, repo_name: &str, message: &str) {
        let repo = self
            .repos
            .iter()
            .find(|r| r.name == repo_name)
            .expect("Repository not found");

        let temp_clone = self.temp_dir.path().join(format!("{}-update", repo_name));
        Command::new("git")
            .args(["clone"])
            .arg(&repo.path)
            .arg(&temp_clone)
            .output()
            .expect("Failed to clone for update");

        Command::new("git")
            .args(["commit", "--allow-empty", "-m", message])
            .current_dir(&temp_clone)
            .output()
            .expect("Failed to create commit");

        Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&temp_clone)
            .output()
            .expect("Failed to push commit");

        std::fs::remove_dir_all(&temp_clone).expect("Failed to remove temp clone");
    }
}

/// Information about a test repository
#[derive(Debug, Clone)]
pub struct TestRepo {
    /// Name of the repository
    pub name: String,
    /// Path to the repository on disk
    pub path: PathBuf,
    /// URL for cloning the repository (file:// URL)
    pub url: String,
}

/// Create a test repository fixture
#[fixture]
pub fn test_workspace() -> TestWorkspace {
    TestWorkspace::new()
}

/// Create a test git repos fixture
#[fixture]
pub fn test_git_repos() -> TestGitRepos {
    TestGitRepos::new()
}

/// Create a workspace config builder with common test setup
pub fn test_workspace_config(base_path: &Path) -> WorkspaceConfigBuilder {
    WorkspaceConfigBuilder::new()
        .worktree_base(base_path.join("worktrees"))
        .default_branch("main".to_string())
}
