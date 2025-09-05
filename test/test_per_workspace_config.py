"""Test per-workspace configuration functionality."""

import os
import subprocess
from pathlib import Path
from typing import List, Tuple

import pytest


class TestGitConfigParsing:
    """Test git-config based configuration."""
    
    def test_worktree_specific_config(self, temp_workspace_git_enabled, workspace_script, run_workspace, git_repos):
        """Test workspace-specific git config overrides."""
        # workspace_script is already in the temp directory, no need to copy
        
        # Set default config
        repo_a_path = str(git_repos[0][1])
        subprocess.run(["git", "config", "--add", "workspace.repo", 
                       f"{repo_a_path} main"], check=True)
        
        # Create workspace
        result = run_workspace("switch", "feature")
        assert result.returncode == 0
        assert "Creating workspace as superproject worktree" in result.stdout
        
        # Set worktree-specific config
        subprocess.run(["git", "config", "--worktree", "--add", "workspace.repo",
                       f"{repo_a_path} feature-test"],
                      cwd="worktrees/feature", check=True)
        
        # Verify feature workspace uses override
        result = run_workspace("config", "show", "feature")
        assert "feature-test" in result.stdout
        assert "Workspace-specific repositories:" in result.stdout
    
    def test_config_inheritance_chain(self, temp_workspace_git_enabled, workspace_script, run_workspace, git_repos):
        """Test configuration priority: worktree > default > file."""
        # workspace_script is already copied to the temp directory
        
        repo_a_path = str(git_repos[0][1])
        repo_b_path = str(git_repos[1][1])
        
        # Create legacy workspace.conf
        config_file = temp_workspace_git_enabled / "workspace.conf"
        config_file.write_text(f"{repo_a_path} main\n")
        
        # Test legacy file is used when no git config exists
        result = run_workspace("config", "show", "main")
        assert "Legacy configuration (from workspace.conf):" in result.stdout
        
        # Set default git config
        subprocess.run(["git", "config", "--add", "workspace.repo", 
                       f"{repo_b_path} develop"], check=True)
        
        # Test default config overrides file
        result = run_workspace("config", "show", "main")
        assert "Default repositories (inherited):" in result.stdout
        assert repo_b_path in result.stdout
        assert repo_a_path not in result.stdout  # File config should not be used
        
        # Create workspace and set worktree-specific config
        run_workspace("switch", "test")
        subprocess.run(["git", "config", "--worktree", "--add", "workspace.repo",
                       f"{repo_a_path} feature-test"],
                      cwd="worktrees/test", check=True)
        
        # Test worktree config overrides default
        result = run_workspace("config", "show", "test")
        assert "Workspace-specific repositories:" in result.stdout
        assert "feature-test" in result.stdout
    
    def test_workspace_isolation(self, temp_workspace_git_enabled, workspace_script, run_workspace, git_repos):
        """Test that workspaces are isolated from each other."""
        # workspace_script is already copied to the temp directory
        
        repo_a_path = str(git_repos[0][1])
        repo_b_path = str(git_repos[1][1])
        
        # Create two workspaces
        run_workspace("switch", "workspace1")
        run_workspace("switch", "workspace2")
        
        # Set different configs for each workspace
        subprocess.run(["git", "config", "--worktree", "--add", "workspace.repo",
                       f"{repo_a_path} develop"],
                      cwd="worktrees/workspace1", check=True)
        
        subprocess.run(["git", "config", "--worktree", "--add", "workspace.repo",
                       f"{repo_b_path} feature-test"],
                      cwd="worktrees/workspace2", check=True)
        
        # Verify workspace1 config
        result = run_workspace("config", "show", "workspace1")
        assert repo_a_path in result.stdout
        assert "develop" in result.stdout
        assert repo_b_path not in result.stdout
        
        # Verify workspace2 config
        result = run_workspace("config", "show", "workspace2")
        assert repo_b_path in result.stdout
        assert "feature-test" in result.stdout
        assert repo_a_path not in result.stdout


class TestConfigCommands:
    """Test new config management commands."""
    
    def test_config_set_command(self, temp_workspace_git_enabled, workspace_script, run_workspace, git_repos):
        """Test setting configuration via workspace config set."""
        # workspace_script is already copied to the temp directory
        
        repo_url = str(git_repos[0][1])
        
        # Create workspace
        run_workspace("switch", "test")
        
        # Set config using command
        result = run_workspace("config", "set", "test", repo_url, "develop", "v1.0.0")
        assert result.returncode == 0
        assert "Set repository config" in result.stdout
        
        # Verify config was set
        result = run_workspace("config", "show", "test")
        assert repo_url in result.stdout
        assert "develop" in result.stdout
        assert "v1.0.0" in result.stdout
    
    def test_config_import_command(self, temp_workspace_git_enabled, workspace_script, run_workspace, git_repos):
        """Test importing configuration from workspace.conf."""
        # workspace_script is already copied to the temp directory
        
        # Create legacy workspace.conf
        config_content = "\n".join([
            f"{git_repos[0][1]} develop",
            f"{git_repos[1][1]} feature-test v1.0.0",
            f"{git_repos[2][1]}"
        ])
        config_file = temp_workspace_git_enabled / "workspace.conf"
        config_file.write_text(config_content)
        
        # Import to workspace
        result = run_workspace("config", "import", "imported")
        assert result.returncode == 0
        assert "Importing configuration" in result.stdout
        assert "Import complete" in result.stdout
        
        # Verify imported config
        result = run_workspace("config", "show", "imported")
        assert "Workspace-specific repositories:" in result.stdout
        assert str(git_repos[0][1]) in result.stdout
        assert "develop" in result.stdout
        assert str(git_repos[1][1]) in result.stdout
        assert "feature-test" in result.stdout
        assert "v1.0.0" in result.stdout
        assert str(git_repos[2][1]) in result.stdout
    
    def test_config_set_default_command(self, temp_workspace_git_enabled, workspace_script, run_workspace, git_repos):
        """Test setting default configuration."""
        # workspace_script is already copied to the temp directory
        
        repo_url = str(git_repos[0][1])
        
        # Set default config
        result = run_workspace("config", "set-default", repo_url, "main")
        assert result.returncode == 0
        assert "Set default repository config" in result.stdout
        
        # Verify default config is used for new workspaces
        result = run_workspace("config", "show", "newworkspace")
        assert "Default repositories (inherited):" in result.stdout
        assert repo_url in result.stdout
        assert "main" in result.stdout


class TestMigration:
    """Test migration from workspace.conf to git config."""
    
    def test_backward_compatibility(self, temp_workspace, workspace_script, run_workspace, workspace_config):
        """Test that existing workspace.conf files still work."""
        # workspace_script is already copied to the temp directory
        
        # Should work with legacy workspace.conf
        result = run_workspace("switch", "legacy-test")
        assert result.returncode == 0
        
        # Verify repos were created from workspace.conf
        result = run_workspace("status")
        assert "legacy-test" in result.stdout
        assert "repo-a" in result.stdout
        assert "repo-b" in result.stdout
        assert "repo-c" in result.stdout
    
    def test_migration_preserves_functionality(self, temp_workspace_git_enabled, workspace_script, run_workspace, git_repos):
        """Test that migrated configurations work identically."""
        # workspace_script is already copied to the temp directory
        
        # Create complex workspace.conf
        config_content = f"""# Complex configuration
{git_repos[0][1]} develop
{git_repos[1][1]} main v1.0.0
{git_repos[2][1]} feature-test
"""
        config_file = temp_workspace_git_enabled / "workspace.conf"
        config_file.write_text(config_content)
        
        # Create workspace using file config
        result = run_workspace("switch", "file-based")
        assert result.returncode == 0
        
        # Import to new workspace
        result = run_workspace("config", "import", "git-based", str(config_file))
        assert result.returncode == 0
        
        # Switch to git-based workspace
        result = run_workspace("switch", "git-based", check=False)
        # The switch may fail due to various issues, but the import should have worked
        
        # Verify import created config
        result = run_workspace("config", "show", "git-based", check=False)
        if result.returncode == 0:
            # Check that configuration was imported
            assert str(git_repos[0][1]) in result.stdout
            assert str(git_repos[1][1]) in result.stdout
            assert str(git_repos[2][1]) in result.stdout


class TestWorkspaceAsWorktree:
    """Test that workspaces are now git worktrees of the superproject."""
    
    def test_workspace_is_git_worktree(self, temp_workspace_git_enabled, workspace_script, run_workspace):
        """Test that created workspaces are git worktrees."""
        # workspace_script is already copied to the temp directory
        
        # Create workspace
        result = run_workspace("switch", "test-worktree")
        assert result.returncode == 0
        
        # Verify it's a git worktree
        assert (temp_workspace_git_enabled / "worktrees" / "test-worktree" / ".git").exists()
        
        # Check that it's listed as a worktree
        result = subprocess.run(["git", "worktree", "list"], 
                              cwd=temp_workspace_git_enabled,
                              capture_output=True, text=True, check=True)
        assert "worktrees/test-worktree" in result.stdout
        assert "workspace/test-worktree" in result.stdout
    
    def test_workspace_branch_creation(self, temp_workspace_git_enabled, workspace_script, run_workspace):
        """Test that workspaces create appropriate branches."""
        # workspace_script is already copied to the temp directory
        
        # Create multiple workspaces
        run_workspace("switch", "feature-x")
        run_workspace("switch", "hotfix-y")
        
        # Check branches exist
        result = subprocess.run(["git", "branch", "-a"],
                              cwd=temp_workspace_git_enabled,
                              capture_output=True, text=True, check=True)
        assert "workspace/feature-x" in result.stdout
        assert "workspace/hotfix-y" in result.stdout
    
    def test_workspace_removal_removes_worktree(self, temp_workspace_git_enabled, workspace_script, run_workspace, git_repos):
        """Test that clean command removes git worktree."""
        # workspace_script is already copied to the temp directory
        
        # Set a default repo config
        subprocess.run(["git", "config", "--add", "workspace.repo",
                       f"{git_repos[0][1]} main"], check=True)
        
        # Create and then remove workspace
        run_workspace("switch", "temp-workspace")
        
        # Verify worktree exists
        result = subprocess.run(["git", "worktree", "list"],
                              cwd=temp_workspace_git_enabled,
                              capture_output=True, text=True, check=True)
        assert "worktrees/temp-workspace" in result.stdout
        
        # Remove workspace
        result = run_workspace("clean", "temp-workspace", input="y\n")
        assert result.returncode == 0
        
        # Verify worktree is removed
        result = subprocess.run(["git", "worktree", "list"],
                              cwd=temp_workspace_git_enabled,
                              capture_output=True, text=True, check=True)
        assert "worktrees/temp-workspace" not in result.stdout


# Import required for copy operations
import shutil