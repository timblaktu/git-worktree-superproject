"""Test suite for the workspace management tool."""

import os
import subprocess
from pathlib import Path

import pytest


class TestWorkspaceInit:
    """Test workspace init command."""
    
    @pytest.mark.parametrize("branch_name,expected_branch", [
        ("main", "main"),
        ("develop", "develop"),
        ("feature-123", "feature-123"),
        ("release/v1.0", "release/v1.0"),
        ("hotfix-urgent", "hotfix-urgent"),
    ])
    def test_init_various_branches(self, run_workspace, workspace_config, clean_workspace, branch_name, expected_branch):
        """Test initializing workspace with various branch names."""
        result = run_workspace("init", branch_name)
        
        assert result.returncode == 0
        assert f"Initializing workspace: {branch_name}" in result.stdout
        assert "Workspace initialized:" in result.stdout
        
        # Check that worktrees directory was created
        worktrees = Path("worktrees")
        assert worktrees.exists()
        assert (worktrees / branch_name).exists()
        
        # Check that all repos were cloned
        workspace = worktrees / branch_name
        assert (workspace / "repo-a").exists()
        assert (workspace / "repo-b").exists()
        assert (workspace / "repo-c").exists()
    
    def test_init_default_branch(self, run_workspace, workspace_config, clean_workspace):
        """Test initializing workspace with default (main) branch."""
        result = run_workspace("init")
        
        assert result.returncode == 0
        assert "Initializing workspace: main" in result.stdout
        assert "Workspace initialized:" in result.stdout
        
        # Check that worktrees directory was created
        worktrees = Path("worktrees")
        assert worktrees.exists()
        assert (worktrees / "main").exists()
        
        # Check that all repos were cloned
        main_workspace = worktrees / "main"
        assert (main_workspace / "repo-a").exists()
        assert (main_workspace / "repo-b").exists()
        assert (main_workspace / "repo-c").exists()
    
    def test_init_custom_branch(self, run_workspace, workspace_config, clean_workspace):
        """Test initializing workspace with custom branch."""
        result = run_workspace("init", "feature-test")
        
        assert result.returncode == 0
        assert "Initializing workspace: feature-test" in result.stdout
        
        # Check workspace was created
        feature_workspace = Path("worktrees/feature-test")
        assert feature_workspace.exists()
        
        # Verify branch in cloned repo
        repo_a_path = feature_workspace / "repo-a"
        branch_result = subprocess.run(
            ["git", "branch", "--show-current"],
            cwd=repo_a_path,
            capture_output=True,
            text=True
        )
        assert branch_result.stdout.strip() == "feature-test"
    
    def test_init_existing_workspace(self, run_workspace, workspace_config, clean_workspace):
        """Test initializing an already existing workspace."""
        # First init
        run_workspace("init", "main")
        
        # Second init should skip existing repos
        result = run_workspace("init", "main")
        assert result.returncode == 0
        assert "exists, skipping" in result.stdout
    
    def test_init_with_pinned_repo(self, run_workspace, complex_workspace_config, clean_workspace):
        """Test initializing workspace with pinned repository."""
        result = run_workspace("init")
        
        assert result.returncode == 0
        
        # Check that repo-c is at the pinned version
        repo_c_path = Path("worktrees/main/repo-c")
        tag_result = subprocess.run(
            ["git", "describe", "--tags"],
            cwd=repo_c_path,
            capture_output=True,
            text=True
        )
        assert tag_result.stdout.strip() == "v1.0.0"
    
    def test_init_no_config(self, run_workspace, temp_workspace, clean_workspace):
        """Test init command without configuration file."""
        # Remove config file
        config_path = temp_workspace / "workspace.conf"
        if config_path.exists():
            config_path.unlink()
        
        result = run_workspace("init", check=False)
        assert result.returncode == 0  # Should succeed with empty workspace


class TestWorkspaceSync:
    """Test workspace sync command."""
    
    def test_sync_current_workspace(self, run_workspace, workspace_config, clean_workspace):
        """Test syncing current workspace."""
        # Initialize workspace first
        run_workspace("init", "main")
        
        # Make a change in original repo
        repos_dir = Path("..").resolve() / "repos"
        repo_a_path = repos_dir / "repo-a"
        subprocess.run(
            ["git", "checkout", "main"],
            cwd=repo_a_path,
            capture_output=True
        )
        (repo_a_path / "new-file.txt").write_text("New content")
        subprocess.run(["git", "add", "."], cwd=repo_a_path, capture_output=True)
        subprocess.run(
            ["git", "commit", "-m", "Add new file"],
            cwd=repo_a_path,
            capture_output=True
        )
        
        # Change to workspace directory and sync
        os.chdir("worktrees/main")
        result = run_workspace("sync")
        
        assert result.returncode == 0
        assert "Syncing workspace: main" in result.stdout
        assert "Updating repo-a" in result.stdout
    
    def test_sync_specific_workspace(self, run_workspace, workspace_config, clean_workspace):
        """Test syncing specific workspace by name."""
        run_workspace("init", "develop")
        
        result = run_workspace("sync", "develop")
        assert result.returncode == 0
        assert "Syncing workspace: develop" in result.stdout
    
    def test_sync_pinned_repo_skipped(self, run_workspace, complex_workspace_config, clean_workspace):
        """Test that pinned repositories are skipped during sync."""
        run_workspace("init")
        
        os.chdir("worktrees/main")
        result = run_workspace("sync")
        
        assert result.returncode == 0
        assert "repo-c is pinned, skipping" in result.stdout
    
    def test_sync_nonexistent_workspace(self, run_workspace, workspace_config, clean_workspace):
        """Test syncing non-existent workspace."""
        result = run_workspace("sync", "nonexistent", check=False)
        
        assert result.returncode != 0
        assert "Workspace not found" in result.stdout


class TestWorkspaceStatus:
    """Test workspace status command."""
    
    def test_status_empty(self, run_workspace, workspace_config, clean_workspace):
        """Test status when no workspaces exist."""
        result = run_workspace("status")
        
        assert result.returncode == 0
        assert "Workspace Status" in result.stdout
        assert "No workspaces found" in result.stdout
    
    def test_status_single_workspace(self, run_workspace, workspace_config, clean_workspace):
        """Test status with single workspace."""
        run_workspace("init")
        
        result = run_workspace("status")
        assert result.returncode == 0
        assert "main" in result.stdout
        assert "repo-a:" in result.stdout
        assert "repo-b:" in result.stdout
        assert "repo-c:" in result.stdout
        assert "[clean]" in result.stdout
    
    def test_status_modified_repo(self, run_workspace, workspace_config, clean_workspace):
        """Test status shows modified repositories."""
        run_workspace("init")
        
        # Modify a file in repo-a
        repo_a_path = Path("worktrees/main/repo-a")
        (repo_a_path / "modified.txt").write_text("Modified content")
        
        result = run_workspace("status")
        assert result.returncode == 0
        assert "[modified]" in result.stdout
    
    def test_status_multiple_workspaces(self, run_workspace, workspace_config, clean_workspace):
        """Test status with multiple workspaces."""
        run_workspace("init", "main")
        run_workspace("init", "develop")
        
        result = run_workspace("status")
        assert result.returncode == 0
        assert "main" in result.stdout
        assert "develop" in result.stdout


class TestWorkspaceForeach:
    """Test workspace foreach command."""
    
    def test_foreach_simple_command(self, run_workspace, workspace_config, clean_workspace):
        """Test executing simple command in all repos."""
        run_workspace("init")
        
        os.chdir("worktrees/main")
        result = run_workspace("foreach", "pwd")
        
        assert result.returncode == 0
        assert "=== repo-a ===" in result.stdout
        assert "=== repo-b ===" in result.stdout
        assert "=== repo-c ===" in result.stdout
        assert "repo-a" in result.stdout
        assert "repo-b" in result.stdout
        assert "repo-c" in result.stdout
    
    def test_foreach_git_command(self, run_workspace, workspace_config, clean_workspace):
        """Test executing git command in all repos."""
        run_workspace("init")
        
        os.chdir("worktrees/main")
        result = run_workspace("foreach", "git", "status", "-s")
        
        assert result.returncode == 0
        assert "=== repo-a ===" in result.stdout
        assert "=== repo-b ===" in result.stdout
        assert "=== repo-c ===" in result.stdout
    
    def test_foreach_not_in_workspace(self, run_workspace, workspace_config, clean_workspace):
        """Test foreach command when not in workspace directory."""
        result = run_workspace("foreach", "pwd", check=False)
        
        assert result.returncode != 0
        assert "Not in workspace" in result.stdout
    
    def test_foreach_with_environment_variables(self, run_workspace, workspace_config, clean_workspace):
        """Test foreach command provides environment variables."""
        run_workspace("init")
        
        os.chdir("worktrees/main")
        result = run_workspace("foreach", "echo name=$name path=$path")
        
        assert result.returncode == 0
        assert "name=repo-a path=repo-a" in result.stdout
        assert "name=repo-b path=repo-b" in result.stdout
        assert "name=repo-c path=repo-c" in result.stdout
    
    def test_foreach_quiet_mode(self, run_workspace, workspace_config, clean_workspace):
        """Test foreach command with --quiet flag."""
        run_workspace("init")
        
        os.chdir("worktrees/main")
        result = run_workspace("foreach", "--quiet", "pwd")
        
        assert result.returncode == 0
        assert "=== repo-a ===" not in result.stdout
        assert "repo-a" in result.stdout  # Should still show command output
    
    @pytest.mark.parametrize("flags", ["-q", "--quiet"])
    def test_foreach_quiet_flags(self, run_workspace, workspace_config, clean_workspace, flags):
        """Test foreach command with different quiet flag formats."""
        run_workspace("init")
        
        os.chdir("worktrees/main")
        result = run_workspace("foreach", flags, "echo test")
        
        assert result.returncode == 0
        assert "===" not in result.stdout
        assert "test" in result.stdout
    
    @pytest.mark.parametrize("command,expected_in_output", [
        # Test shell expansions
        ("echo *.txt", "*.txt"),  # Should show literal when no matches
        ("echo $HOME", "/"),  # Should expand environment variables
        ("echo $(pwd)", "repo-"),  # Should execute subshells
        # Test quoting
        ("echo 'single quotes'", "single quotes"),
        ('echo "double quotes"', "double quotes"),
        ("echo mixed 'single' and \"double\"", "mixed single and double"),
        # Test special characters
        ("echo test > /dev/null && echo success", "success"),
        ("echo line1; echo line2", "line1"),  # Should see both lines
    ])
    def test_foreach_shell_features(self, run_workspace, workspace_config, clean_workspace, command, expected_in_output):
        """Test foreach with various shell features."""
        run_workspace("init")
        
        os.chdir("worktrees/main")
        result = run_workspace("foreach", command)
        
        assert result.returncode == 0
        assert expected_in_output in result.stdout


class TestWorkspaceList:
    """Test workspace list command."""
    
    def test_list_empty(self, run_workspace, workspace_config, clean_workspace):
        """Test listing when no workspaces exist."""
        result = run_workspace("list")
        
        assert result.returncode == 0
        assert "Available Workspaces" in result.stdout
        assert "No workspaces found" in result.stdout
    
    def test_list_single_workspace(self, run_workspace, workspace_config, clean_workspace):
        """Test listing with single workspace."""
        run_workspace("init")
        
        result = run_workspace("list")
        assert result.returncode == 0
        assert "main" in result.stdout
    
    def test_list_multiple_workspaces(self, run_workspace, workspace_config, clean_workspace):
        """Test listing with multiple workspaces."""
        run_workspace("init", "main")
        run_workspace("init", "develop")
        run_workspace("init", "feature-xyz")
        
        result = run_workspace("list")
        assert result.returncode == 0
        assert "main" in result.stdout
        assert "develop" in result.stdout
        assert "feature-xyz" in result.stdout


class TestWorkspaceClean:
    """Test workspace clean command."""
    
    def test_clean_with_confirmation(self, run_workspace, workspace_config, clean_workspace):
        """Test cleaning workspace with confirmation."""
        run_workspace("init", "test-branch")
        
        # Confirm deletion
        result = run_workspace("clean", "test-branch", input="y\n")
        
        assert result.returncode == 0
        assert "Delete workspace: test-branch?" in result.stdout
        assert "Workspace removed" in result.stdout
        
        # Verify workspace is gone
        assert not Path("worktrees/test-branch").exists()
    
    def test_clean_cancelled(self, run_workspace, workspace_config, clean_workspace):
        """Test cancelling workspace cleanup."""
        run_workspace("init", "test-branch")
        
        # Cancel deletion
        result = run_workspace("clean", "test-branch", input="n\n")
        
        assert result.returncode == 0
        assert "Delete workspace: test-branch?" in result.stdout
        
        # Verify workspace still exists
        assert Path("worktrees/test-branch").exists()
    
    def test_clean_nonexistent_workspace(self, run_workspace, workspace_config, clean_workspace):
        """Test cleaning non-existent workspace."""
        result = run_workspace("clean", "nonexistent", check=False)
        
        assert result.returncode != 0
        assert "Workspace not found" in result.stdout
    
    def test_clean_no_name(self, run_workspace, workspace_config, clean_workspace):
        """Test clean command without workspace name."""
        result = run_workspace("clean", check=False)
        
        assert result.returncode != 0
        assert "Workspace name required" in result.stdout


class TestWorkspaceHelp:
    """Test workspace help command."""
    
    def test_help_command(self, run_workspace, workspace_config):
        """Test help command output."""
        result = run_workspace("help")
        
        assert result.returncode == 0
        assert "Workspace Manager" in result.stdout
        assert "Usage:" in result.stdout
        assert "Commands:" in result.stdout
        assert "init" in result.stdout
        assert "sync" in result.stdout
        assert "status" in result.stdout
        assert "foreach" in result.stdout
        assert "list" in result.stdout
        assert "clean" in result.stdout
    
    def test_no_args_shows_help(self, run_workspace, workspace_config):
        """Test that no arguments shows help."""
        result = run_workspace()
        
        assert result.returncode == 0
        assert "Workspace Manager" in result.stdout
        assert "Usage:" in result.stdout
    
    @pytest.mark.parametrize("invalid_command", [
        "invalid",
        "unknown",
        "test",
        "--help",  # Currently shows help but could be improved
    ])
    def test_invalid_commands_show_help(self, run_workspace, workspace_config, invalid_command):
        """Test that invalid commands show help."""
        result = run_workspace(invalid_command)
        
        assert result.returncode == 0
        assert "Workspace Manager" in result.stdout
        assert "Usage:" in result.stdout


class TestStatusVariations:
    """Test status command with various repository states."""
    
    def test_status_with_uncommitted_changes(self, run_workspace, workspace_config, clean_workspace):
        """Test status shows correct state for repos with uncommitted changes."""
        run_workspace("init")
        
        # Create different types of changes
        workspace_path = Path("worktrees/main")
        
        # Untracked file in repo-a
        (workspace_path / "repo-a" / "untracked.txt").write_text("untracked")
        
        # Staged change in repo-b
        (workspace_path / "repo-b" / "staged.txt").write_text("staged")
        subprocess.run(["git", "add", "staged.txt"], cwd=workspace_path / "repo-b", capture_output=True)
        
        # Modified tracked file in repo-c
        readme_path = workspace_path / "repo-c" / "README.md"
        readme_path.write_text("Modified content")
        subprocess.run(["git", "add", "README.md"], cwd=workspace_path / "repo-c", capture_output=True)
        subprocess.run(["git", "commit", "-m", "Add README"], cwd=workspace_path / "repo-c", capture_output=True)
        readme_path.write_text("Modified again")
        
        result = run_workspace("status")
        assert result.returncode == 0
        assert "[modified]" in result.stdout
        assert result.stdout.count("[modified]") >= 2  # At least repo-a and repo-c
    
    @pytest.mark.parametrize("num_workspaces", [0, 1, 3, 5])
    def test_status_with_varying_workspace_counts(self, run_workspace, workspace_config, clean_workspace, num_workspaces):
        """Test status output with different numbers of workspaces."""
        workspace_names = [f"workspace-{i}" for i in range(num_workspaces)]
        
        for name in workspace_names:
            run_workspace("init", name)
        
        result = run_workspace("status")
        assert result.returncode == 0
        
        if num_workspaces == 0:
            assert "No workspaces found" in result.stdout
        else:
            for name in workspace_names:
                assert name in result.stdout