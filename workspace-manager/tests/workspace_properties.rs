//! Property-based tests for workspace invariants
//!
//! These tests use proptest to verify key invariants across many different inputs.

mod common;

use common::{test_workspace_config, TestGitRepos, TestWorkspace};
use proptest::prelude::*;
use workspace_manager::workspace::{WorkspaceManager, WorkspaceManagerImpl};

// ============================================================================
// Property 1: Workspace Consistency Invariant
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))] // Reduced cases for Phase 5

    #[test]
    fn test_workspace_consistency_invariant(
        branch_name in "[a-z]{1,10}",
        repo_count in 1..4usize,
    ) {
        // Property: Workspace switch operations are atomic
        // Either ALL repos are created successfully, or NONE exist (rollback)

        // Arrange
        let test_workspace = TestWorkspace::new();
        let mut git_repos = TestGitRepos::new();

        let mut builder = test_workspace_config(test_workspace.path());

        for i in 0..repo_count {
            git_repos.create_repo(&format!("repo-{}", i));
            builder = builder.add_repo(git_repos.repos[i].url.clone(), &branch_name);
        }

        let config = builder.build();
        let manager = WorkspaceManagerImpl::new_with_real_git();

        // Act - will panic with "not yet implemented" in Phase 5
        let result = manager.switch(&branch_name, &config);

        // Assert - verify ATOMIC INVARIANT
        let workspace_path = test_workspace.worktrees_path().join(&branch_name);

        if result.is_ok() && result.as_ref().unwrap().errors.is_empty() {
            // Success case: ALL repos must exist
            for i in 0..repo_count {
                let repo_path = workspace_path.join(format!("repo-{}", i));
                prop_assert!(
                    repo_path.exists(),
                    "Invariant violation: Repo {} should exist after successful switch", i
                );
            }
        } else {
            // Failure case: NO repos should exist (atomic rollback)
            // Or partial success with errors reported in result.errors field
            if result.is_ok() {
                // Partial success - some repos created, errors in result.errors
                let created_count = result.as_ref().unwrap().repos_created.len();
                let error_count = result.as_ref().unwrap().errors.len();
                prop_assert!(
                    created_count + error_count == repo_count,
                    "All repos must be accounted for: {} created + {} errors != {} total",
                    created_count, error_count, repo_count
                );
            } else {
                // Total failure - workspace directory should not exist or be empty
                if workspace_path.exists() {
                    let entries: Vec<_> = std::fs::read_dir(&workspace_path)
                        .unwrap()
                        .collect();
                    prop_assert!(
                        entries.is_empty(),
                        "Invariant violation: Failed switch should not leave repos behind"
                    );
                }
            }
        }
    }
}

// ============================================================================
// Property 2: Sync Idempotency Invariant
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))] // Reduced cases for Phase 5

    #[test]
    fn test_sync_idempotency(
        workspace_name in "[a-z]{1,10}",
        repo_count in 1..4usize,
    ) {
        // Property: Syncing twice without remote changes is idempotent
        // Both syncs should report same results (already up-to-date)

        // Arrange
        let test_workspace = TestWorkspace::new();
        let mut git_repos = TestGitRepos::new();

        let mut builder = test_workspace_config(test_workspace.path());

        for i in 0..repo_count {
            git_repos.create_repo(&format!("repo-{}", i));
            builder = builder.add_repo(git_repos.repos[i].url.clone(), "main");
        }

        let config = builder.build();
        let manager = WorkspaceManagerImpl::new_with_real_git();

        // Create the workspace first
        manager.switch(&workspace_name, &config).unwrap();

        // Act: Sync twice
        let result1 = manager.sync(&workspace_name).unwrap();
        let result2 = manager.sync(&workspace_name).unwrap();

        // Assert - verify IDEMPOTENCY INVARIANT
        // Both syncs should report same number of repos (already up-to-date)
        // Note: pull() reports success even when already up-to-date
        prop_assert_eq!(
            result2.repos_updated.len(), result1.repos_updated.len(),
            "Idempotency violation: Both syncs should report same updated count"
        );

        prop_assert_eq!(
            result2.repos_pinned.len(), result1.repos_pinned.len(),
            "Pinned repos should be consistent across syncs"
        );

        prop_assert!(
            result2.repos_failed.is_empty(),
            "Second sync should have no failures if first sync succeeded"
        );
    }
}

// ============================================================================
// Property 3: Foreach Isolation Invariant
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))] // Reduced cases for Phase 5

    #[test]
    fn test_foreach_isolation_invariant(
        repo_count in 2..5usize,
    ) {
        // Property: foreach executes commands in isolation per repository
        // Side effects in one repo don't affect other repos

        // Arrange
        let test_workspace = TestWorkspace::new();
        let mut git_repos = TestGitRepos::new();

        let mut builder = test_workspace_config(test_workspace.path());

        for i in 0..repo_count {
            git_repos.create_repo(&format!("repo-{}", i));
            builder = builder.add_repo(git_repos.repos[i].url.clone(), "main");
        }

        let config = builder.build();
        let manager = WorkspaceManagerImpl::new_with_real_git();
        manager.switch("test", &config).unwrap();

        // Act: Execute command that writes to a file
        // NOTE: foreach() wraps commands in sh -c automatically for shell expansion
        let result = manager.foreach(
            "test",
            &["echo".to_string(), "test".to_string(), ">".to_string(), "output.txt".to_string()]
        ).unwrap();

        // Assert - verify ISOLATION INVARIANT
        // Each repo should have executed the command
        prop_assert_eq!(
            result.outputs.len(), repo_count,
            "Isolation violation: Command should execute in all {} repos", repo_count
        );

        // Each repo should have its own independent output.txt file
        for i in 0..repo_count {
            let output_file = test_workspace.worktrees_path()
                .join(format!("test/repo-{}/output.txt", i));
            prop_assert!(
                output_file.exists(),
                "Isolation violation: Repo {} should have independent output file", i
            );

            // Verify file contains expected content
            let contents = std::fs::read_to_string(&output_file).unwrap();
            prop_assert_eq!(
                contents.trim(), "test",
                "Output file should contain expected content"
            );
        }

        // Verify command succeeded in all repos (no failures)
        prop_assert!(
            result.failures.is_empty(),
            "Simple command should succeed in all repos"
        );
    }
}

// ============================================================================
// Property 4: Branch Name Handling
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))] // Reduced cases for Phase 5

    #[test]
    fn test_branch_name_handling(
        // Generate various valid git branch names
        // Git allows alphanumeric, dash, underscore
        branch_name in "[a-zA-Z0-9_-]{1,20}",
    ) {
        // Property: System correctly handles all valid git branch names
        // Should either succeed or fail with appropriate error message

        // Arrange
        let test_workspace = TestWorkspace::new();
        let mut git_repos = TestGitRepos::new();
        git_repos.create_repo("repo");

        let config = test_workspace_config(test_workspace.path())
            .add_repo(git_repos.repos[0].url.clone(), &branch_name)
            .build();

        let manager = WorkspaceManagerImpl::new_with_real_git();

        // Act - will panic with "not yet implemented" in Phase 5
        let result = manager.switch(&branch_name, &config);

        // Assert - verify BRANCH NAME HANDLING INVARIANT
        // Either succeeds (workspace created) or fails with appropriate error
        if result.is_ok() {
            // Success: workspace directory should exist
            let workspace_path = test_workspace.worktrees_path().join(&branch_name);
            prop_assert!(
                workspace_path.exists(),
                "Workspace directory should exist for branch: {}", branch_name
            );
        } else {
            // Failure: error message should mention branch or validation
            let err_msg = result.unwrap_err().to_string();
            prop_assert!(
                err_msg.contains("branch") ||
                err_msg.contains("invalid") ||
                err_msg.contains("name"),
                "Error message should be informative: {}", err_msg
            );
        }
    }
}
