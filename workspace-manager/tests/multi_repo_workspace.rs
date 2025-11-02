//! Integration tests for multi-repository workspace management
//!
//! These tests use real git operations in isolated temporary directories
//! to verify the complete workspace lifecycle.

mod common;

use common::{test_git_repos, test_workspace, test_workspace_config, TestGitRepos, TestWorkspace};
use rstest::*;
use workspace_manager::workspace::{WorkspaceManager, WorkspaceManagerImpl};

// ============================================================================
// Category 1: Core Lifecycle Tests
// ============================================================================

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace switch")]
fn test_switch_creates_workspace_with_multiple_repos(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
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

    // Act - will panic with "not yet implemented"
    let _result = manager.switch("main", &config).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert_eq!(result.repos_created.len(), 3);
    // assert!(test_workspace.worktrees_path().join("main/repo-a").exists());
    // assert!(test_workspace.worktrees_path().join("main/repo-b").exists());
    // assert!(test_workspace.worktrees_path().join("main/repo-c").exists());
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace switch")]
fn test_switch_existing_workspace_is_idempotent(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - first switch creates workspace
    let _result1 = manager.switch("main", &config).unwrap();

    // Act - second switch should skip existing repos
    let _result2 = manager.switch("main", &config).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert_eq!(result1.repos_created.len(), 1);
    // assert_eq!(result1.repos_skipped.len(), 0);
    // assert_eq!(result2.repos_created.len(), 0);
    // assert_eq!(result2.repos_skipped.len(), 1);
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace switch")]
fn test_switch_with_pinned_repository(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Arrange
    test_git_repos.create_repo("repo-a");
    test_git_repos.create_repo_with_tag("repo-b", "v1.0.0");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .add_pinned_repo(test_git_repos.repos[1].url.clone(), "v1.0.0")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented"
    let _result = manager.switch("main", &config).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // - repo-a is on main branch
    // - repo-b is checked out at tag v1.0.0
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace switch")]
fn test_switch_with_different_branch_names(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "develop")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented"
    let _result = manager.switch("develop", &config).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // - Workspace named "develop" is created
    // - Repository is checked out at "develop" branch
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace remove")]
fn test_remove_workspace_deletes_all_repos(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    let _result = manager.switch("test-workspace", &config).unwrap();

    // Act - will panic with "not yet implemented"
    manager.remove("test-workspace").unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert!(!test_workspace.worktrees_path().join("test-workspace").exists());
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

    // Act - will panic with "not yet implemented"
    let _result = manager.sync("main").unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert_eq!(result.repos_updated.len(), 2);
    // assert!(result.repos_updated.contains(&"repo-a".to_string()));
    // assert!(result.repos_updated.contains(&"repo-b".to_string()));
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace sync")]
fn test_sync_skips_pinned_repos_integration(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
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

    // Act - will panic with "not yet implemented"
    let _result = manager.sync("main").unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert_eq!(result.repos_updated.len(), 1);
    // assert_eq!(result.repos_pinned.len(), 1);
    // assert_eq!(result.repos_pinned[0], "repo-b");
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace sync")]
fn test_sync_nonexistent_workspace_errors(test_workspace: TestWorkspace) {
    // Arrange
    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented"
    let result = manager.sync("nonexistent");

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert!(result.is_err());
    // assert!(result.unwrap_err().to_string().contains("not found"));
}

// ============================================================================
// Category 3: Bulk Operations Tests
// ============================================================================

#[rstest]
#[should_panic(expected = "Phase 6: Implement foreach command")]
fn test_foreach_executes_command_in_all_repos(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Arrange
    test_git_repos.create_repo("repo-a");
    test_git_repos.create_repo("repo-b");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .add_repo(test_git_repos.repos[1].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Act - will panic with "not yet implemented"
    let _result = manager.foreach("main", &["pwd".to_string()]).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert_eq!(result.outputs.len(), 2);
    // assert!(result.outputs[0].stdout.contains("repo-a"));
    // assert!(result.outputs[1].stdout.contains("repo-b"));
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement foreach command")]
fn test_foreach_provides_environment_variables(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Act - will panic with "not yet implemented"
    let _result = manager
        .foreach("main", &["echo".to_string(), "$name".to_string()])
        .unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert_eq!(result.outputs[0].stdout.trim(), "repo-a");
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement foreach command")]
fn test_foreach_handles_command_failures(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Act - command that fails
    let _result = manager.foreach("main", &["false".to_string()]).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert_eq!(result.outputs[0].exit_code, 1);
}

// ============================================================================
// Category 4: State Inspection Tests
// ============================================================================

#[rstest]
#[should_panic(expected = "Phase 6: Implement status command")]
fn test_status_reflects_actual_git_state(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Act - will panic with "not yet implemented"
    let _result = manager.status(Some("main".to_string())).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert_eq!(result.workspaces.len(), 1);
    // assert!(matches!(result.workspaces[0].repos[0].status, RepoStatus::Clean { .. }));
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace list")]
fn test_list_returns_all_workspaces(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();
    manager.switch("develop", &config).unwrap();

    // Act - will panic with "not yet implemented"
    let _result = manager.list().unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert_eq!(result.len(), 2);
    // assert!(result.iter().any(|w| w.name == "main"));
    // assert!(result.iter().any(|w| w.name == "develop"));
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace list")]
fn test_list_empty_returns_empty_vec(test_workspace: TestWorkspace) {
    // Arrange
    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented"
    let _result = manager.list().unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert_eq!(result.len(), 0);
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement status command")]
fn test_status_detects_modified_files(
    test_workspace: TestWorkspace,
    mut test_git_repos: TestGitRepos,
) {
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

    // Act - will panic with "not yet implemented"
    let _result = manager.status(Some("main".to_string())).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert!(matches!(result.workspaces[0].repos[0].status,
    //     RepoStatus::Modified { .. } | RepoStatus::Untracked { .. }));
}

// ============================================================================
// Category 5: Error Handling Tests
// ============================================================================

#[rstest]
#[should_panic(expected = "Phase 6: Implement git status operations")]
fn test_broken_worktree_detection(test_workspace: TestWorkspace) {
    // Arrange - create a broken worktree manually
    let workspace_path = test_workspace.worktrees_path().join("main/repo-a");
    std::fs::create_dir_all(&workspace_path).unwrap();

    // Create invalid .git file
    std::fs::write(workspace_path.join(".git"), "gitdir: /nonexistent/path\n").unwrap();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented"
    let _result = manager.status(Some("main".to_string())).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert!(matches!(result.workspaces[0].repos[0].status,
    //     RepoStatus::Broken { .. }));
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement git status operations")]
fn test_uninitialized_repo_handling(test_workspace: TestWorkspace) {
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

    // Act - will panic with "not yet implemented"
    let _result = manager.status(Some("main".to_string())).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert!(matches!(result.workspaces[0].repos[0].status,
    //     RepoStatus::Uninitialized));
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement git status operations")]
fn test_detached_head_handling(test_workspace: TestWorkspace, mut test_git_repos: TestGitRepos) {
    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.switch("main", &config).unwrap();

    // Detach HEAD
    let repo_path = test_workspace.worktrees_path().join("main/repo-a");
    std::process::Command::new("git")
        .args(["checkout", "HEAD^0"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Act - will panic with "not yet implemented"
    let _result = manager.status(Some("main".to_string())).unwrap();

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify:
    // assert!(matches!(result.workspaces[0].repos[0].status,
    //     RepoStatus::DetachedHead { .. }));
}

#[rstest]
#[should_panic(expected = "Phase 6: Implement workspace switch")]
fn test_partial_failure_rollback(test_workspace: TestWorkspace, mut test_git_repos: TestGitRepos) {
    // Arrange
    test_git_repos.create_repo("repo-a");

    let config = test_workspace_config(test_workspace.path())
        .add_repo(test_git_repos.repos[0].url.clone(), "main")
        .add_repo("file:///nonexistent/repo", "main") // This will fail
        .build();

    let manager = WorkspaceManagerImpl::new_with_real_git();

    // Act - will panic with "not yet implemented"
    let result = manager.switch("main", &config);

    // Assert (unreachable in Phase 4)
    // In Phase 6, this will verify atomic rollback:
    // assert!(result.is_err() || result.unwrap().errors.len() > 0);
    // Verify workspace is in consistent state (all or nothing)
}
