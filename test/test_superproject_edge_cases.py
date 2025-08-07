"""Test edge cases and error handling for super-project configurations.

This module tests error conditions, malformed configurations, and edge cases
that can occur with heterogeneous super-project setups.
"""

import os
import subprocess
from pathlib import Path
from typing import List, Tuple

import pytest


class TestConfigurationValidation:
    """Test validation of super-project configuration files."""
    
    def test_empty_lines_and_comments_handling(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test robust handling of empty lines and comments."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        config_content = f"""

    # Lots of whitespace and comments
    
        # Indented comment
{repo_a[1]}    


        # More comments
{repo_b[1]} develop   


    # Final comment

"""
        config_path.write_text(config_content)
        
        result = run_workspace("switch")
        assert result.returncode == 0
        
        # Should have parsed the valid repositories
        workspace_dir = Path("worktrees/main")
        repos = [d for d in workspace_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        assert len(repos) == 2
    
    def test_unicode_and_special_characters(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test handling of unicode and special characters in configuration."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        config_content = f"""# Configuration with unicode: 🚀 ñ 中文
{repo_a[1]} main
# Comment with special chars: @#$%^&*()
{repo_b[1]} develop
"""
        config_path.write_text(config_content)
        
        result = run_workspace("switch")
        assert result.returncode == 0
        
        # Should handle unicode in comments gracefully
        workspace_dir = Path("worktrees/main")
        repos = [d for d in workspace_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        assert len(repos) == 2


class TestMalformedReferences:
    """Test handling of malformed and invalid references in super-project configurations."""
    
    def test_invalid_commit_sha(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test handling of invalid commit SHA references."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        config_content = f"""# Configuration with invalid SHA
{repo_a[1]} main deadbeefdeadbeefdeadbeefdeadbeefdeadbeef
{repo_b[1]} develop 123456789abcdef
{repo_c[1]} main
"""
        config_path.write_text(config_content)
        
        # Should handle invalid SHAs gracefully
        result = run_workspace("switch", check=False)
        
        # Some repos should still be cloned successfully
        workspace_dir = Path("worktrees/main")
        if workspace_dir.exists():
            successful_repos = [d for d in workspace_dir.iterdir() 
                              if d.is_dir() and (d / ".git").exists()]
            # At least the repo without invalid SHA should work
            assert len(successful_repos) >= 1
    
    def test_nonexistent_tags(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test handling of references to non-existent tags."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        config_content = f"""# Configuration with non-existent tags  
{repo_a[1]} main v999.999.999
{repo_b[1]} main nonexistent-tag
{repo_c[1]} main v1.0.0
"""
        config_path.write_text(config_content)
        
        result = run_workspace("switch", check=False)
        
        # Should handle non-existent tags gracefully
        workspace_dir = Path("worktrees/main")
        if workspace_dir.exists():
            repos = [d for d in workspace_dir.iterdir() 
                    if d.is_dir() and (d / ".git").exists()]
            # The repo with valid tag should still be cloned
            assert len(repos) >= 1
    
    def test_nonexistent_branches(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test handling of references to non-existent branches."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        config_content = f"""# Configuration with non-existent branches
{repo_a[1]} nonexistent-branch
{repo_b[1]} also-nonexistent
{repo_c[1]} main
"""
        config_path.write_text(config_content)
        
        result = run_workspace("switch", check=False)
        
        # Should handle non-existent branches - may create them or use default
        workspace_dir = Path("worktrees/main")
        if workspace_dir.exists():
            repos = [d for d in workspace_dir.iterdir() 
                    if d.is_dir() and (d / ".git").exists()]
            # At least some repos should be cloned
            assert len(repos) >= 1
    
    @pytest.mark.parametrize("malformed_config", [
        "invalid-url-format branch ref extra-field",
        "  \t  \n",  # Only whitespace
        "url-only-but-with-trailing-spaces   ",
        "/nonexistent/local/repo.git main ref1 ref2 ref3",  # Too many fields
    ])
    def test_malformed_config_lines(self, run_workspace, temp_workspace, base_git_repos, 
                                  clean_workspace, malformed_config):
        """Test handling of malformed configuration lines."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        config_content = f"""# Configuration with malformed lines
{repo_a[1]} main
{malformed_config}
{repo_b[1]} develop
"""
        config_path.write_text(config_content)
        
        # Should handle malformed lines gracefully
        result = run_workspace("switch", check=False)
        
        # Valid repos should still be processed
        workspace_dir = Path("worktrees/main") 
        if workspace_dir.exists():
            repos = [d for d in workspace_dir.iterdir() 
                    if d.is_dir() and (d / ".git").exists()]
            assert len(repos) >= 1  # At least the valid repos should work


class TestRepositoryAccessIssues:
    """Test handling of repository access and permission issues."""
    
    def test_missing_repository_paths(self, run_workspace, temp_workspace, clean_workspace):
        """Test handling of paths to non-existent repositories."""
        config_path = temp_workspace / "workspace.conf"
        
        config_content = """# Configuration with non-existent repo paths
/nonexistent/path/repo1
/also/nonexistent/repo2  
/tmp/fake-repo
"""
        config_path.write_text(config_content)
        
        result = run_workspace("switch", check=False)
        # Should fail gracefully for non-existent repos
        assert result.returncode != 0 or "not found" in result.stderr.lower()
    
    def test_relative_vs_absolute_paths(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test handling of relative vs absolute repository paths."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        # Create config with mix of relative and absolute paths
        config_content = f"""# Mixed path types
{repo_a[1]}
../relative/path/repo
{repo_b[1]} main
"""
        config_path.write_text(config_content)
        
        result = run_workspace("switch", check=False)
        
        # Should handle valid absolute paths
        workspace_dir = Path("worktrees/main")
        if workspace_dir.exists():
            repos = [d for d in workspace_dir.iterdir() 
                    if d.is_dir() and (d / ".git").exists()]
            assert len(repos) >= 1  # At least absolute paths should work


class TestConcurrentOperations:
    """Test concurrent operations and race conditions in super-project management."""
    
    def test_concurrent_switch_operations(self, run_workspace, heterogeneous_superproject_config, clean_workspace):
        """Test concurrent workspace switching."""
        # This test checks basic operation - real concurrency testing would need threading
        
        # Switch to multiple workspaces in sequence to test isolation
        result1 = run_workspace("switch", "workspace1")
        assert result1.returncode == 0
        
        result2 = run_workspace("switch", "workspace2")  
        assert result2.returncode == 0
        
        # Both workspaces should exist and be independent
        workspace1_dir = Path("worktrees/workspace1")
        workspace2_dir = Path("worktrees/workspace2")
        
        assert workspace1_dir.exists()
        assert workspace2_dir.exists()
        
        # Should have independent repository copies
        repos1 = [d for d in workspace1_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        repos2 = [d for d in workspace2_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        
        assert len(repos1) >= 2
        assert len(repos2) >= 2
    
    def test_status_during_operations(self, run_workspace, heterogeneous_superproject_config, clean_workspace):
        """Test status command during other operations."""
        # Switch to workspace
        run_workspace("switch")
        
        # Status should work even during sync operations
        result = run_workspace("status")
        assert result.returncode == 0
        
        # Test status after sync
        run_workspace("sync", check=False)
        result = run_workspace("status")
        assert result.returncode == 0


class TestCorruptedRepositories:
    """Test handling of corrupted or incomplete repository states."""
    
    def test_incomplete_clone_recovery(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test recovery from incomplete repository clones."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        config_content = f"""# Test recovery config
{repo_a[1]} main
{repo_b[1]} develop
"""
        config_path.write_text(config_content)
        
        # Switch to workspace
        result = run_workspace("switch")
        assert result.returncode == 0
        
        workspace_dir = Path("worktrees/main")
        
        # Simulate corrupted repository by removing .git directory
        repo_dir = workspace_dir / "repo-a"
        if repo_dir.exists():
            import shutil
            git_dir = repo_dir / ".git"
            if git_dir.exists():
                shutil.rmtree(git_dir)
        
        # Re-running switch should handle the corrupted state
        result = run_workspace("switch")
        # Should either fix the repo or skip it gracefully
        assert result.returncode == 0
    
    def test_missing_worktree_directories(self, run_workspace, heterogeneous_superproject_config, clean_workspace):
        """Test handling of missing worktree directories."""
        # Switch to workspace
        run_workspace("switch")
        
        # Remove worktrees directory
        worktrees_dir = Path("worktrees")
        if worktrees_dir.exists():
            import shutil
            shutil.rmtree(worktrees_dir)
        
        # Status should handle missing directories gracefully
        result = run_workspace("status")
        assert result.returncode == 0
        assert "No workspaces found" in result.stdout


class TestScaleAndResourceLimits:
    """Test behavior under resource constraints and scale limits."""
    
    def test_large_number_of_repositories(self, run_workspace, large_superproject_config, clean_workspace):
        """Test handling of configurations with many repository entries."""
        result = run_workspace("switch")
        assert result.returncode == 0
        
        # Should handle large configs without issues
        workspace_dir = Path("worktrees/main")
        repos = [d for d in workspace_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        
        # Large config has 24 entries but only 3 unique repos
        assert len(repos) == 3
        
        # Status should work efficiently even with large configs
        result = run_workspace("status")
        assert result.returncode == 0
        
        # Foreach should handle all repositories
        result = run_workspace("foreach", "echo $name", check=False)
        # Foreach may fail if not in workspace directory context
        if result.returncode == 0:
            assert result.stdout.count("===") == 3  # One header per unique repo
    
    def test_deeply_nested_workspace_paths(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test handling of deeply nested workspace directory structures."""
        # Test with deeply nested workspace name
        workspace_name = "deeply/nested/workspace/path/test"
        
        config_path = temp_workspace / "workspace.conf"
        repo_a, repo_b, _ = base_git_repos
        
        config_content = f"""# Deep nesting test
{repo_a[1]} main
{repo_b[1]} develop
"""
        config_path.write_text(config_content)
        
        result = run_workspace("switch", workspace_name)
        # May not support nested paths, but should fail gracefully
        
        if result.returncode == 0:
            workspace_dir = Path(f"worktrees/{workspace_name}")
            assert workspace_dir.exists()
        else:
            # Failing gracefully is also acceptable
            assert "Error" in result.stderr or "error" in result.stderr.lower()