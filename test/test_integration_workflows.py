"""Test complex integration workflows simulating real-world usage."""

import os
import shutil
import tempfile
from pathlib import Path
import pytest
import subprocess
import time


class TestDevelopmentWorkflows:
    """Test real-world development workflows across multiple repositories."""

    def test_feature_development_workflow(self, temp_workspace_git_enabled, git_repos, run_workspace):
        """Test complete feature development workflow."""
        workspace_dir = temp_workspace_git_enabled
        
        # Step 1: Create feature workspace
        result = run_workspace("switch", "feature-xyz", check=False)
        assert result.returncode == 0
        
        # Step 2: Make changes in multiple repos
        repo_a_path = workspace_dir / "worktrees" / "feature-xyz" / "repo-a"
        repo_b_path = workspace_dir / "worktrees" / "feature-xyz" / "repo-b"
        
        # Add feature file to repo-a
        (repo_a_path / "feature-a.txt").write_text("Feature A implementation")
        subprocess.run(["git", "add", "."], cwd=repo_a_path, check=True)
        subprocess.run(["git", "commit", "-m", "Add feature A"], cwd=repo_a_path, check=True)
        
        # Add feature file to repo-b
        (repo_b_path / "feature-b.txt").write_text("Feature B implementation")
        subprocess.run(["git", "add", "."], cwd=repo_b_path, check=True)
        subprocess.run(["git", "commit", "-m", "Add feature B"], cwd=repo_b_path, check=True)
        
        # Step 3: Check status
        status_result = run_workspace("status", check=False)
        assert status_result.returncode == 0
        assert "feature-xyz" in status_result.stdout
        
        # Step 4: Push changes
        subprocess.run(["git", "push", "origin", "feature-xyz"], cwd=repo_a_path, check=True)
        subprocess.run(["git", "push", "origin", "feature-xyz"], cwd=repo_b_path, check=True)
        
        # Step 5: Switch to main for hotfix
        result = run_workspace("switch", "main", check=False)
        assert result.returncode == 0
        
        # Verify feature files don't exist in main
        assert not (workspace_dir / "worktrees" / "main" / "repo-a" / "feature-a.txt").exists()
        assert not (workspace_dir / "worktrees" / "main" / "repo-b" / "feature-b.txt").exists()
        
        # Step 6: Make hotfix
        hotfix_file = workspace_dir / "worktrees" / "main" / "repo-a" / "hotfix.txt"
        hotfix_file.write_text("Critical fix")
        subprocess.run(["git", "add", "."], cwd=hotfix_file.parent, check=True)
        subprocess.run(["git", "commit", "-m", "Critical hotfix"], cwd=hotfix_file.parent, check=True)
        subprocess.run(["git", "push", "origin", "main"], cwd=hotfix_file.parent, check=True)
        
        # Step 7: Switch back to feature and sync
        result = run_workspace("switch", "feature-xyz", check=False)
        assert result.returncode == 0
        
        # Merge main into feature
        subprocess.run(["git", "fetch", "origin"], cwd=repo_a_path, check=True)
        subprocess.run(["git", "merge", "origin/main"], cwd=repo_a_path, check=True)
        
        # Both feature and hotfix should exist
        assert (repo_a_path / "feature-a.txt").exists()
        assert (repo_a_path / "hotfix.txt").exists()

    def test_release_management_workflow(self, temp_workspace_git_enabled, git_repos, run_workspace):
        """Test release management with version pinning."""
        workspace_dir = temp_workspace_git_enabled
        
        # Step 1: Create release branch
        result = run_workspace("switch", "release-1.0", check=False)
        assert result.returncode == 0
        
        # Step 2: Tag repositories
        repo_a_path = workspace_dir / "worktrees" / "release-1.0" / "repo-a"
        repo_b_path = workspace_dir / "worktrees" / "release-1.0" / "repo-b"
        
        subprocess.run(["git", "tag", "v1.0.0"], cwd=repo_a_path, check=True)
        subprocess.run(["git", "tag", "v1.0.0"], cwd=repo_b_path, check=True)
        subprocess.run(["git", "push", "origin", "v1.0.0"], cwd=repo_a_path, check=True)
        subprocess.run(["git", "push", "origin", "v1.0.0"], cwd=repo_b_path, check=True)
        
        # Step 3: Configure pinned workspace for production
        result = run_workspace(
            ["config", "set", "production", str(git_repos["repo-a"]), "", "v1.0.0"],
            workspace_dir
        )
        assert result.returncode == 0
        
        result = run_workspace(
            "config", "set", "production", str(git_repos["repo-b"]), "", "v1.0.0",
            check=False
        )
        assert result.returncode == 0
        
        # Step 4: Create production workspace with pinned versions
        result = run_workspace("switch", "production", check=False)
        assert result.returncode == 0
        
        # Step 5: Verify repos are at tagged versions
        prod_repo_a = workspace_dir / "worktrees" / "production" / "repo-a"
        tag_result = subprocess.run(
            ["git", "describe", "--tags", "--exact-match"],
            cwd=prod_repo_a,
            capture_output=True,
            text=True
        )
        assert "v1.0.0" in tag_result.stdout
        
        # Step 6: Try to sync (should skip pinned repos)
        sync_result = run_workspace("sync", "production", check=False)
        assert sync_result.returncode == 0
        # Should indicate pinned repos were skipped

    def test_parallel_development_workflow(self, temp_workspace_git_enabled, git_repos, run_workspace):
        """Test parallel development on different features."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create multiple feature workspaces
        features = ["feature-auth", "feature-ui", "feature-api"]
        
        for feature in features:
            result = run_workspace("switch", feature, check=False)
            assert result.returncode == 0
            
            # Make unique changes in each workspace
            repo_path = workspace_dir / "worktrees" / feature / "repo-a"
            feature_file = repo_path / f"{feature}.txt"
            feature_file.write_text(f"Implementation for {feature}")
            subprocess.run(["git", "add", "."], cwd=repo_path, check=True)
            subprocess.run(["git", "commit", "-m", f"Implement {feature}"], cwd=repo_path, check=True)
        
        # List all workspaces
        list_result = run_workspace("list", check=False)
        assert list_result.returncode == 0
        for feature in features:
            assert feature in list_result.stdout
        
        # Check status of all workspaces
        status_result = run_workspace("status", check=False)
        assert status_result.returncode == 0
        # Should show all workspaces and their states
        
        # Switch between workspaces rapidly
        for _ in range(2):
            for feature in features:
                result = run_workspace("switch", feature, check=False)
                assert result.returncode == 0
                # Verify correct feature file exists
                feature_file = workspace_dir / "worktrees" / feature / "repo-a" / f"{feature}.txt"
                assert feature_file.exists()

    def test_team_collaboration_workflow(self, temp_workspace_git_enabled, git_repos, run_workspace):
        """Test team collaboration with shared configurations."""
        workspace_dir = temp_workspace_git_enabled
        
        # Developer 1: Set up shared configuration
        shared_config = workspace_dir / "team.conf"
        shared_config.write_text(f"""
repo {git_repos["repo-a"]} develop
repo {git_repos["repo-b"]} develop
""")
        
        # Import shared configuration
        result = run_workspace("config", "import", "team-dev", str(shared_config), check=False)
        assert result.returncode == 0
        
        # Developer 1: Create workspace
        result = run_workspace("switch", "team-dev", check=False)
        assert result.returncode == 0
        
        # Developer 1: Make changes
        dev1_file = workspace_dir / "worktrees" / "team-dev" / "repo-a" / "dev1-feature.txt"
        dev1_file.write_text("Developer 1 feature")
        subprocess.run(["git", "add", "."], cwd=dev1_file.parent, check=True)
        subprocess.run(["git", "commit", "-m", "Dev1 feature"], cwd=dev1_file.parent, check=True)
        subprocess.run(["git", "push", "origin", "develop"], cwd=dev1_file.parent, check=True)
        
        # Simulate Developer 2 in different workspace
        dev2_workspace = Path(tempfile.mkdtemp())
        try:
            # Initialize git in dev2 workspace
            subprocess.run(["git", "init"], cwd=dev2_workspace, check=True)
            
            # Copy shared configuration
            shutil.copy(shared_config, dev2_workspace / "team.conf")
            
            # Import configuration
            result = run_workspace(
                ["config", "import", "team-dev", str(dev2_workspace / "team.conf")],
                dev2_workspace
            )
            assert result.returncode == 0
            
            # Create workspace
            result = run_workspace(["switch", "team-dev"], dev2_workspace)
            assert result.returncode == 0
            
            # Sync to get Dev1's changes
            result = run_workspace(["sync", "team-dev"], dev2_workspace)
            assert result.returncode == 0
            
            # Verify Dev1's changes are visible
            dev1_file_dev2 = dev2_workspace / "worktrees" / "team-dev" / "repo-a" / "dev1-feature.txt"
            assert dev1_file_dev2.exists()
            
        finally:
            shutil.rmtree(dev2_workspace)

    def test_ci_cd_integration_workflow(self, temp_workspace_git_enabled, git_repos, run_workspace):
        """Test CI/CD integration patterns."""
        workspace_dir = temp_workspace_git_enabled
        
        # CI: Create workspace for testing
        result = run_workspace("switch", "ci-test", check=False)
        assert result.returncode == 0
        
        # CI: Run tests in all repositories
        test_results = []
        result = run_workspace(
            ["foreach", "echo 'Running tests in $name'"],
            workspace_dir
        )
        assert result.returncode == 0
        
        # CD: Create deployment workspace with specific versions
        deploy_config = workspace_dir / "deploy.conf"
        
        # Get current commit hashes for pinning
        repo_a_path = workspace_dir / "worktrees" / "ci-test" / "repo-a"
        commit_a = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=repo_a_path,
            capture_output=True,
            text=True
        ).stdout.strip()
        
        repo_b_path = workspace_dir / "worktrees" / "ci-test" / "repo-b"
        commit_b = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=repo_b_path,
            capture_output=True,
            text=True
        ).stdout.strip()
        
        # Configure deployment with exact commits
        deploy_config.write_text(f"""
repo {git_repos["repo-a"]} @pinned {commit_a}
repo {git_repos["repo-b"]} @pinned {commit_b}
""")
        
        # Import deployment configuration
        result = run_workspace("config", "import", "deploy", str(deploy_config), check=False)
        assert result.returncode == 0
        
        # Create deployment workspace
        result = run_workspace("switch", "deploy", check=False)
        assert result.returncode == 0
        
        # Verify exact commits are checked out
        deploy_repo_a = workspace_dir / "worktrees" / "deploy" / "repo-a"
        deploy_commit_a = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=deploy_repo_a,
            capture_output=True,
            text=True
        ).stdout.strip()
        assert deploy_commit_a == commit_a


class TestComplexScenarios:
    """Test complex scenarios and edge cases in workflows."""

    def test_large_scale_repository_management(self, temp_workspace_git_enabled, run_workspace):
        """Test managing many repositories efficiently."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create configuration with many repositories
        many_repos = []
        for i in range(10):
            repo_path = Path(tempfile.mkdtemp()) / f"repo-{i}"
            repo_path.mkdir(parents=True)
            subprocess.run(["git", "init"], cwd=repo_path, check=True)
            (repo_path / "README.md").write_text(f"Repository {i}")
            subprocess.run(["git", "add", "."], cwd=repo_path, check=True)
            subprocess.run(["git", "commit", "-m", "Initial"], cwd=repo_path, check=True)
            many_repos.append(repo_path)
        
        # Create configuration
        config_lines = [f"repo {repo}" for repo in many_repos]
        config_file = workspace_dir / "many.conf"
        config_file.write_text("\n".join(config_lines))
        
        # Import configuration
        result = run_workspace("config", "import", "large-scale", str(config_file), check=False)
        assert result.returncode == 0
        
        # Create workspace (should handle many repos efficiently)
        start_time = time.time()
        result = run_workspace("switch", "large-scale", check=False)
        creation_time = time.time() - start_time
        assert result.returncode == 0
        
        # Verify all repositories were created
        for i in range(10):
            repo_path = workspace_dir / "worktrees" / "large-scale" / f"repo-{i}"
            assert repo_path.exists()
        
        # Test foreach command on many repos
        result = run_workspace(
            ["foreach", "echo Processing $name"],
            workspace_dir
        )
        assert result.returncode == 0
        for i in range(10):
            assert f"repo-{i}" in result.stdout
        
        # Performance check (should be reasonably fast)
        assert creation_time < 60  # Should complete within 1 minute

    def test_mixed_repository_types(self, temp_workspace_git_enabled, git_repos, run_workspace):
        """Test workspace with mixed repository types (HTTPS, SSH, local)."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create local repository
        local_repo = Path(tempfile.mkdtemp()) / "local-repo"
        local_repo.mkdir(parents=True)
        subprocess.run(["git", "init"], cwd=local_repo, check=True)
        (local_repo / "local.txt").write_text("Local repository")
        subprocess.run(["git", "add", "."], cwd=local_repo, check=True)
        subprocess.run(["git", "commit", "-m", "Initial"], cwd=local_repo, check=True)
        
        # Create mixed configuration
        config_file = workspace_dir / "mixed.conf"
        config_file.write_text(f"""
# HTTPS repository
repo {git_repos["repo-a"]}

# Local path repository
repo {local_repo}

# File URL repository  
repo file://{local_repo}
""")
        
        # Import configuration
        result = run_workspace("config", "import", "mixed", str(config_file), check=False)
        assert result.returncode == 0
        
        # Create workspace with mixed types
        result = run_workspace("switch", "mixed", check=False)
        assert result.returncode == 0
        
        # Verify all repository types work
        assert (workspace_dir / "worktrees" / "mixed" / "repo-a").exists()
        assert (workspace_dir / "worktrees" / "mixed" / "local-repo").exists()

    def test_workspace_recovery_from_errors(self, temp_workspace_git_enabled, git_repos, run_workspace):
        """Test recovery from various error conditions."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create workspace
        result = run_workspace("switch", "recovery-test", check=False)
        assert result.returncode == 0
        
        # Simulate partial failure by corrupting one worktree
        repo_a_path = workspace_dir / "worktrees" / "recovery-test" / "repo-a"
        git_file = repo_a_path / ".git"
        if git_file.exists():
            git_file.write_text("corrupted")
        
        # Try to sync (should handle corrupted worktree)
        sync_result = run_workspace("sync", "recovery-test", check=False)
        # Should complete with error for corrupted repo
        
        # Try to clean and recreate
        clean_result = run_workspace("clean", "recovery-test", check=False, input="y\n")
        
        # Recreate workspace
        result = run_workspace("switch", "recovery-test", check=False)
        assert result.returncode == 0
        
        # Should be working again
        assert (workspace_dir / "worktrees" / "recovery-test" / "repo-a").exists()

    def test_long_running_workspace_lifecycle(self, temp_workspace_git_enabled, git_repos, run_workspace):
        """Test long-running workspace with many operations."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create long-lived workspace
        result = run_workspace("switch", "long-lived", check=False)
        assert result.returncode == 0
        
        # Simulate many development cycles
        repo_path = workspace_dir / "worktrees" / "long-lived" / "repo-a"
        
        for cycle in range(5):
            # Add files
            (repo_path / f"cycle{cycle}.txt").write_text(f"Cycle {cycle} content")
            subprocess.run(["git", "add", "."], cwd=repo_path, check=True)
            subprocess.run(["git", "commit", "-m", f"Cycle {cycle}"], cwd=repo_path, check=True)
            
            # Sync
            result = run_workspace("sync", "long-lived", check=False)
            assert result.returncode == 0
            
            # Check status
            result = run_workspace("status", check=False)
            assert result.returncode == 0
        
        # Verify workspace is still healthy after many operations
        assert (workspace_dir / "worktrees" / "long-lived").exists()
        
        # Verify git history is intact
        log_result = subprocess.run(
            ["git", "log", "--oneline"],
            cwd=repo_path,
            capture_output=True,
            text=True
        )
        for cycle in range(5):
            assert f"Cycle {cycle}" in log_result.stdout

    def test_migration_workflow(self, temp_workspace_git_enabled, run_workspace):
        """Test migrating from legacy to new configuration system."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create legacy workspace.conf
        legacy_config = workspace_dir / "workspace.conf"
        legacy_config.write_text("""
# Legacy configuration
repo https://example.com/legacy1.git main
repo https://example.com/legacy2.git develop
repo https://example.com/legacy3.git @pinned v1.0
""")
        
        # Create workspace using legacy config
        result = run_workspace("switch", "legacy-workspace", check=False)
        # Should work with legacy config
        
        # Migrate to new config system
        result = run_workspace("config", "import", "migrated", check=False)
        assert result.returncode == 0
        
        # Verify migration
        show_result = run_workspace("config", "show", "migrated", check=False)
        assert result.returncode == 0
        
        # Create new workspace with migrated config
        result = run_workspace("switch", "migrated", check=False)
        # Should use new config system
        
        # Both workspaces should coexist
        list_result = run_workspace("list", check=False)
        assert "legacy-workspace" in list_result.stdout
        assert "migrated" in list_result.stdout