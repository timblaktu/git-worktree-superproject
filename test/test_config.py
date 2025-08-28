"""Test configuration parsing for the workspace management tool."""

import subprocess
from pathlib import Path

import pytest


class TestConfigurationParsing:
    """Test configuration file parsing.
    
    Uses class-scoped fixtures for better performance since these tests
    only read configuration files without modifying repositories.
    """
    
    def test_simple_config(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test parsing simple configuration."""
        config_path = temp_workspace / "workspace.conf"
        config_content = "\n".join([
            "# Simple configuration",
            f"{base_git_repos[0][1]}",
            f"{base_git_repos[1][1]}",
            ""  # Ensure trailing newline
        ])
        config_path.write_text(config_content)
        
        result = run_workspace("switch")
        assert result.returncode == 0
        
        # Check only specified repos were cloned
        workspace_dir = Path("worktrees/main")
        assert (workspace_dir / "repo-a").exists()
        assert (workspace_dir / "repo-b").exists()
        assert not (workspace_dir / "repo-c").exists()
    
    def test_config_with_branches(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test configuration with specific branches."""
        config_path = temp_workspace / "workspace.conf"
        config_content = "\n".join([
            f"{base_git_repos[0][1]}",
            f"{base_git_repos[1][1]} develop",
            f"{base_git_repos[2][1]} feature-test",
            ""  # Ensure trailing newline
        ])
        config_path.write_text(config_content)
        
        result = run_workspace("switch", "main")
        assert result.returncode == 0
        
        # Verify branches
        workspace_dir = Path("worktrees/main")
        
        # repo-a should be on main
        branch_a = subprocess.run(
            ["git", "branch", "--show-current"],
            cwd=workspace_dir / "repo-a",
            capture_output=True,
            text=True
        )
        assert branch_a.stdout.strip() == "main"
        
        # repo-b should be on develop
        branch_b = subprocess.run(
            ["git", "branch", "--show-current"],
            cwd=workspace_dir / "repo-b",
            capture_output=True,
            text=True
        )
        assert branch_b.stdout.strip() == "develop"
        
        # repo-c should be on feature-test
        branch_c = subprocess.run(
            ["git", "branch", "--show-current"],
            cwd=workspace_dir / "repo-c",
            capture_output=True,
            text=True
        )
        assert branch_c.stdout.strip() == "feature-test"
    
    def test_config_with_refs(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test configuration with specific refs (tags/commits)."""
        config_path = temp_workspace / "workspace.conf"
        config_content = "\n".join([
            f"{base_git_repos[0][1]}",
            f"{base_git_repos[1][1]} main",
            f"{base_git_repos[2][1]} main v1.0.0",
            ""  # Ensure trailing newline
        ])
        config_path.write_text(config_content)
        
        result = run_workspace("switch")
        assert result.returncode == 0
        
        # Verify repo-c is at tag v1.0.0
        workspace_dir = Path("worktrees/main")
        tag_result = subprocess.run(
            ["git", "describe", "--tags"],
            cwd=workspace_dir / "repo-c",
            capture_output=True,
            text=True
        )
        assert tag_result.stdout.strip() == "v1.0.0"
    
    def test_config_comments_and_empty_lines(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test that comments and empty lines are ignored."""
        config_path = temp_workspace / "workspace.conf"
        config_content = "\n".join([
            "# This is a comment",
            "",
            f"{base_git_repos[0][1]}",
            "# Another comment",
            "   # Indented comment",
            "",
            f"{base_git_repos[1][1]}",
            "  ",  # Whitespace-only line
            f"{base_git_repos[2][1]}",
            ""  # Ensure trailing newline
        ])
        config_path.write_text(config_content)
        
        result = run_workspace("switch")
        assert result.returncode == 0
        
        # All repos should be cloned
        workspace_dir = Path("worktrees/main")
        assert (workspace_dir / "repo-a").exists()
        assert (workspace_dir / "repo-b").exists()
        assert (workspace_dir / "repo-c").exists()
    
    @pytest.mark.parametrize("config_content,test_description", [
        ("", "empty configuration file"),
        ("\n\n\n", "only newlines"),
        ("# Just comments here\n# No actual repos\n\n# More comments\n", "only comments"),
        ("  \t  \n  \t\n", "only whitespace"),
        ("# Comment\n\n# Another comment\n\n\n", "comments and empty lines"),
    ])
    def test_empty_or_comment_only_configs(self, run_workspace, temp_workspace, clean_workspace,
                                           config_content, test_description):
        """Test configurations with no actual repository entries."""
        config_path = temp_workspace / "workspace.conf"
        config_path.write_text(config_content)
        
        result = run_workspace("switch")
        # Should succeed but create empty workspace
        assert result.returncode == 0, f"Failed for {test_description}"
        assert Path("worktrees/main").exists(), f"Workspace not created for {test_description}"


class TestConfigurationErrors:
    """Test configuration error handling."""
    
    def test_missing_config_file(self, run_workspace, temp_workspace, clean_workspace):
        """Test behavior when config file is missing."""
        config_path = temp_workspace / "workspace.conf"
        if config_path.exists():
            config_path.unlink()
        
        result = run_workspace("switch", check=False)
        assert result.returncode == 0  # Should succeed with empty workspace
    
    def test_invalid_git_url(self, run_workspace, temp_workspace, clean_workspace):
        """Test handling of invalid git URLs."""
        config_path = temp_workspace / "workspace.conf"
        config_content = "not-a-valid-git-url\n"
        config_path.write_text(config_content)
        
        result = run_workspace("switch", check=False)
        assert result.returncode != 0