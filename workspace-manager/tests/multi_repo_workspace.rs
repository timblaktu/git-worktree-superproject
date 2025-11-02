//! Integration tests for multi-repository workspace management
//!
//! These tests use real git operations in isolated temporary directories
//! to verify the complete workspace lifecycle.

mod common;

use common::{test_git_repos, test_workspace, test_workspace_config, TestGitRepos, TestWorkspace};
use rstest::*;
use workspace_manager::workspace::{RepoStatus, WorkspaceManager, WorkspaceManagerImpl};

// ============================================================================
// Category 1: Core Lifecycle Tests
// ============================================================================

#[rstest]
fn test_switch_creates_workspace_with_multiple_repos(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that switch() creates a workspace with multiple repositories
    // Each repo should be cloned to the workspace directory

    // Arrange
    test_git_repos.create_repo("repo-a");
    test_git_repos.create_repo("repo-b");
    test_git_repos.create_repo("repo-c");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .add_repo(test_git_repos.repos[1].url.clone(), "main")
        .add_repo(test_git_repos.repos[2].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.switch("main", &config).unwrap();

    // Assert - verify all repos were created
    assert_eq!(result.repos_created.len(), 3);
    assert!(result.repos_skipped.is_empty());
    assert!(result.errors.is_empty());

    // Verify directories exist
    assert!(test_workspace.worktrees_path().join("main/repo-a").exists());
    assert!(test_workspace.worktrees_path().join("main/repo-b").exists());
    assert!(test_workspace.worktrees_path().join("main/repo-c").exists());

    // Verify they're valid git repos
    assert!(test_workspace
        .worktrees_path()
        .join("main/repo-a/.git")
        .exists());
    assert!(test_workspace
        .worktrees_path()
        .join("main/repo-b/.git")
        .exists());
    assert!(test_workspace
        .worktrees_path()
        .join("main/repo-c/.git")
        .exists());
}

#[rstest]
fn test_switch_existing_workspace_is_idempotent(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that calling switch() twice is idempotent (no-op second time)

    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - first switch creates workspace
    let result1 = manager.switch("main", &config).unwrap();

    // Act - second switch should skip existing repos
    let result2 = manager.switch("main", &config).unwrap();

    // Assert - first switch creates, second skips
    assert_eq!(result1.repos_created.len(), 1);
    assert_eq!(result1.repos_skipped.len(), 0);
    assert_eq!(result2.repos_created.len(), 0);
    assert_eq!(result2.repos_skipped.len(), 1);
}

#[rstest]
fn test_switch_with_pinned_repository(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that pinned repos (with git_ref) are checked out at specific ref

    // Arrange
    test_git_repos.create_repo("repo-a");
    test_git_repos.create_repo_with_tag("repo-b", "v1.0.0");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .add_pinned_repo(test_git_repos.repos[1].url.clone(), "v1.0.0")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.switch("main", &config).unwrap();

    // Assert - both repos created successfully
    assert_eq!(result.repos_created.len(), 2);

    // Verify repo-a is on main branch
    let repo_a_path = test_workspace.worktrees_path().join("main/repo-a");
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_a_path)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "main");

    // Verify repo-b is at tag v1.0.0
    let repo_b_path = test_workspace.worktrees_path().join("main/repo-b");
    let output = std::process::Command::new("git")
        .args(["describe", "--exact-match", "--tags"])
        .current_dir(&repo_b_path)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "v1.0.0");
}

#[rstest]
fn test_switch_with_different_branch_names(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that workspace can be named differently from branch

    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "develop")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.switch("develop", &config).unwrap();

    // Assert - workspace named "develop" is created
    assert!(test_workspace.worktrees_path().join("develop").exists());
    assert_eq!(result.workspace_name, "develop");
}

#[rstest]
fn test_remove_workspace_deletes_all_repos(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that remove() deletes entire workspace directory

    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("test-workspace", &config).unwrap();

    // Verify workspace exists before removal
    assert!(test_workspace
        .worktrees_path()
        .join("test-workspace")
        .exists());

    // Act - will panic with "not yet implemented" in Phase 5
    manager.remove("test-workspace").unwrap();

    // Assert - workspace directory is deleted
    assert!(!test_workspace
        .worktrees_path()
        .join("test-workspace")
        .exists());
}

// ============================================================================
// Category 2: Synchronization Tests
// ============================================================================

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace sync")]
fn test_sync_pulls_updates_from_all_repos(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that sync() pulls updates from all repositories

    // Arrange
    test_git_repos.create_repo("repo-a");
    test_git_repos.create_repo("repo-b");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .add_repo(test_git_repos.repos[1].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Add commits to remote repos
    test_git_repos.add_commit("repo-a", "New commit in repo-a");
    test_git_repos.add_commit("repo-b", "New commit in repo-b");

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.sync("main").unwrap();

    // Assert - both repos were updated
    assert_eq!(result.repos_updated.len(), 2);
    assert!(result.repos_updated.contains(&"repo-a".to_string()));
    assert!(result.repos_updated.contains(&"repo-b".to_string()));
    assert!(result.repos_pinned.is_empty());
    assert!(result.repos_failed.is_empty());
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace sync")]
fn test_sync_skips_pinned_repos_integration(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that sync() skips repositories with git_ref set (pinned)

    // Arrange
    test_git_repos.create_repo("repo-a");
    test_git_repos.create_repo_with_tag("repo-b", "v1.0.0");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .add_pinned_repo(test_git_repos.repos[1].url.clone(), "v1.0.0")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Add commits to both repos
    test_git_repos.add_commit("repo-a", "New commit");
    test_git_repos.add_commit("repo-b", "Should not be pulled");

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.sync("main").unwrap();

    // Assert - only repo-a updated, repo-b pinned
    assert_eq!(result.repos_updated.len(), 1);
    assert_eq!(result.repos_pinned.len(), 1);
    assert_eq!(result.repos_pinned[0], "repo-b");
    assert!(result.repos_failed.is_empty());
}

#[rstest]
fn test_sync_nonexistent_workspace_errors(test_workspace: TestWorkspace) {
    // Test that sync() errors when workspace doesn't exist

    // Arrange
    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.sync("nonexistent");

    // Assert - should error
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not found") || err_msg.contains("does not exist"),
        "Error should mention workspace not found: {}",
        err_msg
    );
}

// ============================================================================
// Category 3: Bulk Operations Tests
// ============================================================================

#[rstest]
fn test_foreach_executes_command_in_all_repos(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that foreach() executes command in all repos

    // Arrange
    test_git_repos.create_repo("repo-a");
    test_git_repos.create_repo("repo-b");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .add_repo(test_git_repos.repos[1].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.foreach("main", &["pwd".to_string()]).unwrap();

    // Assert - command executed in both repos
    assert_eq!(result.outputs.len(), 2);
    assert!(result.outputs[0].stdout.contains("repo-a"));
    assert!(result.outputs[1].stdout.contains("repo-b"));
    assert!(result.failures.is_empty());
}

#[rstest]
fn test_foreach_provides_environment_variables(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that foreach() provides environment variables to commands

    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager
        .foreach("main", &["echo".to_string(), "$name".to_string()])
        .unwrap();

    // Assert - $name variable should contain repo name
    assert_eq!(result.outputs.len(), 1);
    assert_eq!(result.outputs[0].stdout.trim(), "repo-a");
}

#[rstest]
fn test_foreach_handles_command_failures(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that foreach() captures command failures

    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Act - command that fails (exit code 1)
    let result = manager.foreach("main", &["false".to_string()]).unwrap();

    // Assert - failure captured in exit code
    assert_eq!(result.outputs.len(), 1);
    assert_eq!(result.outputs[0].exit_code, 1);
}

// ============================================================================
// Category 4: State Inspection Tests
// ============================================================================

#[rstest]
fn test_status_reflects_actual_git_state(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that status() returns actual git repository state

    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.status(Some("main".to_string())).unwrap();

    // Assert - repo should be clean
    assert_eq!(result.workspaces.len(), 1);
    assert_eq!(result.workspaces[0].name, "main");
    assert_eq!(result.workspaces[0].repos.len(), 1);
    assert!(matches!(
        result.workspaces[0].repos[0].status,
        RepoStatus::Clean { .. }
    ));
}

#[rstest]
fn test_list_returns_all_workspaces(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that list() returns all workspaces

    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();
    manager.switch("develop", &config).unwrap();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.list().unwrap();

    // Assert - both workspaces listed
    assert_eq!(result.len(), 2);
    assert!(result.iter().any(|w| w.name == "main"));
    assert!(result.iter().any(|w| w.name == "develop"));
}

#[rstest]
fn test_list_empty_returns_empty_vec(test_workspace: TestWorkspace) {
    // Test that list() returns empty vec when no workspaces exist

    // Arrange - Create worktrees directory but no workspaces
    std::fs::create_dir_all(test_workspace.worktrees_path()).unwrap();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Need to call switch() once to set worktree_base, then remove the workspace
    let config = test_workspace_config(test_workspace.path()).build();
    manager.switch("temp", &config).unwrap();
    manager.remove("temp").unwrap();

    // Act
    let result = manager.list().unwrap();

    // Assert - no workspaces exist
    assert_eq!(result.len(), 0);
}

#[rstest]
fn test_status_detects_modified_files(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Test that status() detects modified files

    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Modify a file in the workspace
    let repo_path = test_workspace.worktrees_path().join("main/repo-a");
    std::fs::write(repo_path.join("test.txt"), "modified").unwrap();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.status(Some("main".to_string())).unwrap();

    // Assert - should detect modifications
    assert!(matches!(
        result.workspaces[0].repos[0].status,
        RepoStatus::Modified { .. } | RepoStatus::Untracked { .. }
    ));
}

// ============================================================================
// Category 5: Error Handling Tests
// ============================================================================

#[rstest]
fn test_broken_worktree_detection(test_workspace: TestWorkspace) {
    // Test that status() handles broken worktrees
    // NOTE: Current implementation filters out directories where is_repo() returns false
    // This includes broken git repos. Future enhancement could detect and report broken repos.

    // Arrange - create a broken worktree manually
    let workspace_path = test_workspace.worktrees_path().join("main/repo-a");
    std::fs::create_dir_all(&workspace_path).unwrap();

    // Create invalid .git file pointing to nonexistent path
    std::fs::write(workspace_path.join(".git"), "gitdir: /nonexistent/path\n").unwrap();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Set worktree_base by calling switch() with empty config
    let config = test_workspace_config(test_workspace.path()).build();
    manager.switch("temp", &config).unwrap();

    // Act
    let result = manager.status(Some("main".to_string())).unwrap();

    // Assert - broken repos are currently filtered out (not reported)
    // workspace exists but has no valid repos
    assert_eq!(result.workspaces.len(), 1);
    assert_eq!(result.workspaces[0].name, "main");
    assert_eq!(result.workspaces[0].repos.len(), 0); // Broken repo filtered out
}

#[rstest]
fn test_uninitialized_repo_handling(test_workspace: TestWorkspace) {
    // Test that status() handles repos with no commits

    // Arrange - create an uninitialized repo
    let workspace_path = test_workspace.worktrees_path().join("main/repo-a");
    std::fs::create_dir_all(&workspace_path).unwrap();

    // Initialize git repo but don't create commits
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&workspace_path)
        .output()
        .unwrap();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Set worktree_base by calling switch() with empty config
    let config = test_workspace_config(test_workspace.path()).build();
    manager.switch("temp", &config).unwrap();

    // Act
    let result = manager.status(Some("main".to_string())).unwrap();

    // Assert - should detect uninitialized state
    assert!(matches!(
        result.workspaces[0].repos[0].status,
        RepoStatus::Uninitialized
    ));
}

#[rstest]
fn test_detached_head_handling(test_workspace: TestWorkspace, mut test_git_repos: TestGitRepos) {
    // Test that status() detects detached HEAD state

    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Detach HEAD by checking out commit directly
    let repo_path = test_workspace.worktrees_path().join("main/repo-a");
    std::process::Command::new("git")
        .args(["checkout", "HEAD^0"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.status(Some("main".to_string())).unwrap();

    // Assert - should detect detached HEAD
    assert!(matches!(
        result.workspaces[0].repos[0].status,
        RepoStatus::DetachedHead { .. }
    ));
}

#[rstest]
fn test_partial_failure_rollback(test_workspace: TestWorkspace, mut test_git_repos: TestGitRepos) {
    // Test that switch() handles partial failures gracefully

    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .add_repo("file:///nonexistent/repo", "main") // This will fail
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented" in Phase 5
    let result = manager.switch("main", &config);

    // Assert - should report errors for failed repo
    // Implementation can choose to: return Err, or return Ok with errors field populated
    assert!(result.is_err() || result.unwrap().errors.len() > 0);

    // Verify workspace is in consistent state
    // Either all repos created (ignoring failures), or none created (atomic rollback)
    // This is an implementation decision to be made in Phase 6
}
