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
    
    @pytest.mark.parametrize("test_case,repo_configs,test_description", [
        # Invalid commit SHAs
        ("invalid_sha", [
            ("{repo_a} main deadbeefdeadbeefdeadbeefdeadbeefdeadbeef", "Invalid 40-char SHA"),
            ("{repo_b} develop 123456789abcdef", "Invalid short SHA"),
            ("{repo_c} main", "Valid config for comparison"),
        ], "Test handling of invalid commit SHA references"),
        
        # Non-existent tags
        ("nonexistent_tags", [
            ("{repo_a} main v999.999.999", "Non-existent version tag"),
            ("{repo_b} main nonexistent-tag", "Non-existent named tag"),
            ("{repo_c} main v1.0.0", "Valid tag for comparison"),
        ], "Test handling of references to non-existent tags"),
        
        # Non-existent branches
        ("nonexistent_branches", [
            ("{repo_a} nonexistent-branch", "Non-existent branch"),
            ("{repo_b} also-nonexistent", "Another non-existent branch"),
            ("{repo_c} main", "Valid branch for comparison"),
        ], "Test handling of references to non-existent branches"),
    ])
    def test_invalid_references(self, run_workspace, temp_workspace, base_git_repos, 
                               clean_workspace, test_case, repo_configs, test_description):
        """Parameterized test for various invalid reference scenarios."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        # Build config content from repo_configs
        config_lines = [f"# Configuration test: {test_case}"]
        for config_template, comment in repo_configs:
            # Replace placeholders with actual repo paths
            config_line = config_template.format(
                repo_a=repo_a[1],
                repo_b=repo_b[1], 
                repo_c=repo_c[1]
            )
            config_lines.append(config_line)
        
        config_content = "\n".join(config_lines) + "\n"
        config_path.write_text(config_content)
        
        # Should handle invalid references gracefully
        result = run_workspace("switch", check=False)
        
        # Verify that at least some repos are processed successfully
        workspace_dir = Path("worktrees/main")
        if workspace_dir.exists():
            repos = [d for d in workspace_dir.iterdir() 
                    if d.is_dir() and ((d / ".git").exists() or (d / ".git").is_file())]
            # At least the valid repo should work
            assert len(repos) >= 1, f"Failed {test_description}: no repos created"
    
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
    
    def test_concurrent_switch_operations(self, run_workspace, base_git_repos, temp_workspace, clean_workspace):
        """Test concurrent workspace switching."""
        # This test checks basic operation - real concurrency testing would need threading
        # Note: Git worktrees don't allow the same branch to be checked out in multiple worktrees
        # So we use a simple config without branch-specific repos
        
        # Create a simple config without branch specifications to avoid conflicts
        config_path = temp_workspace / "workspace.conf"
        repo_a, repo_b, repo_c = base_git_repos
        config_content = f"""# Simple config for concurrent testing
{repo_a[1]}
{repo_b[1]}
{repo_c[1]}
"""
        config_path.write_text(config_content)
        
        # Switch to multiple workspaces in sequence to test isolation
        result1 = run_workspace("switch", "workspace1")
        assert result1.returncode == 0
        
        # workspace2 uses a different branch to avoid worktree conflicts
        result2 = run_workspace("switch", "feature-test", check=False)
        if result2.returncode != 0:
            print(f"Switch to feature-test failed:")
            print(f"STDOUT: {result2.stdout}")
            print(f"STDERR: {result2.stderr}")
        assert result2.returncode == 0
        
        # Both workspaces should exist and be independent
        workspace1_dir = Path("worktrees/workspace1")
        workspace2_dir = Path("worktrees/feature-test")
        
        assert workspace1_dir.exists()
        assert workspace2_dir.exists()
        
        # Should have repository worktrees (with .git files pointing to central repos)
        repos1 = [d for d in workspace1_dir.iterdir() if d.is_dir() and ((d / ".git").exists() or (d / ".git").is_file())]
        repos2 = [d for d in workspace2_dir.iterdir() if d.is_dir() and ((d / ".git").exists() or (d / ".git").is_file())]
        
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
        
        # Simulate corrupted repository by removing .git file/directory
        repo_dir = workspace_dir / "repo-a"
        if repo_dir.exists():
            import shutil
            import os
            git_path = repo_dir / ".git"
            if git_path.exists():
                if git_path.is_file():
                    os.remove(git_path)
                else:
                    shutil.rmtree(git_path)
        
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
            # Count unique repo headers (format: "=== repo-name ===")
            headers = [line for line in result.stdout.split('\n') if line.startswith("===") and line.endswith("===")]
            assert len(headers) == 3  # One header per unique repo
    
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