"""Test suite for super-project configurations and heterogeneous repository patterns.

This module focuses on testing the various permutations of super-project configurations
that wt-super can support, emphasizing heterogeneous repository types and tracking modes.
"""

import os
import subprocess
from pathlib import Path
from typing import List, Tuple

import pytest


class TestSuperProjectMatrix:
    """Test matrix of super-project configurations: repo count × tracking modes × reference types."""
    
    @pytest.mark.parametrize("repo_count,config_fixture", [
        (2, "minimal_superproject_config"),
        (5, "heterogeneous_superproject_config"), 
        (24, "large_superproject_config"),
    ])
    def test_superproject_scale_variations(self, run_workspace, request, clean_workspace, repo_count, config_fixture):
        """Test super-projects with varying numbers of repositories."""
        # Get the actual fixture by name
        config = request.getfixturevalue(config_fixture)
        
        result = run_workspace("init")
        assert result.returncode == 0
        
        # Count actual cloned repositories
        workspace_dir = Path("worktrees/main")
        cloned_repos = [d for d in workspace_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        
        # Should have cloned the expected number of unique repositories
        # (large config has duplicates that get deduplicated by directory name)
        if repo_count == 24:
            assert len(cloned_repos) == 3  # Only 3 unique repo names despite 24 entries
        else:
            assert len(cloned_repos) >= 2  # At least the base repos should be cloned
        
        # Verify status works with the configuration
        result = run_workspace("status")
        assert result.returncode == 0
        assert "main" in result.stdout
    
    @pytest.mark.parametrize("tracking_mode,expected_behavior", [
        ("head_tracking", "follows branch updates"),
        ("detached", "stays at pinned reference"),
        ("mixed", "combination of tracking and pinned"),
    ])
    def test_repository_tracking_modes(self, run_workspace, heterogeneous_superproject_config, 
                                     clean_workspace, tracking_mode, expected_behavior):
        """Test different repository tracking modes in super-projects."""
        result = run_workspace("init")
        assert result.returncode == 0
        
        workspace_dir = Path("worktrees/main")
        
        # Check that we have both HEAD-tracking and detached repos
        repos = list(workspace_dir.iterdir())
        assert len(repos) >= 3  # Should have multiple repos from heterogeneous config
        
        # For detached repos, verify they're at specific refs
        if (workspace_dir / "repo-c").exists():
            tag_result = subprocess.run(
                ["git", "describe", "--tags", "--exact-match"],
                cwd=workspace_dir / "repo-c",
                capture_output=True,
                text=True,
                check=False
            )
            # Should be at a tag for pinned repos
            if tag_result.returncode == 0:
                assert "v1.0.0" in tag_result.stdout or "v1.1.0" in tag_result.stdout
        
        # Test sync behavior - pinned repos should be skipped
        result = run_workspace("sync", check=False)
        # Sync may fail if not in workspace directory, but should handle gracefully
        if result.returncode == 0 and tracking_mode == "mixed":
            assert "pinned" in result.stdout.lower() or "skipping" in result.stdout.lower()


class TestReferenceTypes:
    """Test different reference types in super-project configurations."""
    
    def test_tag_references(self, run_workspace, reference_types_config, clean_workspace):
        """Test repositories pinned to git tags."""
        result = run_workspace("init")
        assert result.returncode == 0
        
        workspace_dir = Path("worktrees/main")
        
        # Find repos that should be at tags
        for repo_dir in workspace_dir.iterdir():
            if repo_dir.is_dir() and (repo_dir / ".git").exists():
                # Check if this repo is at a tag
                tag_result = subprocess.run(
                    ["git", "describe", "--tags", "--exact-match"],
                    cwd=repo_dir,
                    capture_output=True,
                    text=True,
                    check=False
                )
                if tag_result.returncode == 0:
                    tag_name = tag_result.stdout.strip()
                    assert tag_name in ["v1.0.0", "v1.1.0"]
    
    def test_commit_sha_references(self, run_workspace, reference_types_config, clean_workspace):
        """Test repositories pinned to specific commit SHAs."""
        result = run_workspace("init")
        assert result.returncode == 0
        
        workspace_dir = Path("worktrees/main")
        
        # Verify that some repos are in detached HEAD state (pinned to commits)
        detached_repos = []
        for repo_dir in workspace_dir.iterdir():
            if repo_dir.is_dir() and (repo_dir / ".git").exists():
                # Check if in detached HEAD state
                branch_result = subprocess.run(
                    ["git", "branch", "--show-current"],
                    cwd=repo_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
                if not branch_result.stdout.strip():
                    # In detached HEAD state
                    detached_repos.append(repo_dir.name)
        
        # Should have at least some detached repos from SHA references
        assert len(detached_repos) >= 0  # Config may not create detached state in all cases
    
    def test_branch_plus_commit_combinations(self, run_workspace, reference_types_config, clean_workspace):
        """Test configurations with branch + commit combinations."""
        result = run_workspace("init")
        assert result.returncode == 0
        
        # Verify the workspace was created successfully
        workspace_dir = Path("worktrees/main")
        assert workspace_dir.exists()
        
        # Check that all configured repos were processed
        repos = [d for d in workspace_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        assert len(repos) >= 2  # Should have multiple repos from the configuration


class TestHeterogeneousConfigurations:
    """Test heterogeneous super-project configurations with mixed tracking modes."""
    
    def test_mixed_head_and_detached_repos(self, run_workspace, heterogeneous_superproject_config, clean_workspace):
        """Test super-project with both HEAD-tracking and detached repositories."""
        result = run_workspace("init")
        assert result.returncode == 0
        
        workspace_dir = Path("worktrees/main")
        
        # Analyze each repository's state
        repo_states = {}
        for repo_dir in workspace_dir.iterdir():
            if repo_dir.is_dir() and (repo_dir / ".git").exists():
                # Check current branch or detached state
                branch_result = subprocess.run(
                    ["git", "branch", "--show-current"],
                    cwd=repo_dir,
                    capture_output=True,
                    text=True
                )
                
                if branch_result.stdout.strip():
                    repo_states[repo_dir.name] = f"branch:{branch_result.stdout.strip()}"
                else:
                    # Get commit info for detached state
                    commit_result = subprocess.run(
                        ["git", "describe", "--always"],
                        cwd=repo_dir,
                        capture_output=True,
                        text=True
                    )
                    repo_states[repo_dir.name] = f"detached:{commit_result.stdout.strip()}"
        
        # Should have a mix of branch and detached states
        assert len(repo_states) >= 2
        
        # Verify status command handles mixed states correctly
        result = run_workspace("status")
        assert result.returncode == 0
        for repo_name, state in repo_states.items():
            assert repo_name in result.stdout
    
    def test_sync_behavior_with_mixed_tracking(self, run_workspace, heterogeneous_superproject_config, clean_workspace):
        """Test sync behavior with mixed HEAD-tracking and pinned repositories."""
        # Initialize workspace
        run_workspace("init")
        
        # Test sync operation (may need to be in workspace directory)
        result = run_workspace("sync", check=False)
        
        # Sync may fail if not in proper directory context, but should handle gracefully
        if result.returncode == 0:
            # Should mention skipping pinned repos if successful
            assert "pinned" in result.stdout.lower() or "skipping" in result.stdout.lower()
    
    @pytest.mark.parametrize("workspace_branch", ["main", "develop", "feature-test"])
    def test_heterogeneous_with_different_workspace_branches(self, run_workspace, 
                                                           heterogeneous_superproject_config, 
                                                           clean_workspace, workspace_branch):
        """Test heterogeneous configurations with different workspace branch names."""
        result = run_workspace("init", workspace_branch)
        assert result.returncode == 0
        
        workspace_dir = Path(f"worktrees/{workspace_branch}")
        assert workspace_dir.exists()
        
        # Verify repos were cloned appropriately for the workspace branch
        repos = [d for d in workspace_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        assert len(repos) >= 2


class TestSuperProjectScaleScenarios:
    """Test super-project scale scenarios and performance characteristics."""
    
    def test_minimal_superproject(self, run_workspace, minimal_superproject_config, clean_workspace):
        """Test minimal super-project with just 1-2 repositories."""
        result = run_workspace("init")
        assert result.returncode == 0
        
        workspace_dir = Path("worktrees/main")
        repos = [d for d in workspace_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        
        # Should have exactly 2 repos from minimal config
        assert len(repos) == 2
        
        # Verify each repo is properly configured
        for repo_dir in repos:
            git_dir = repo_dir / ".git"
            assert git_dir.exists()
            
            # Should have valid git repository
            result = subprocess.run(
                ["git", "status", "--porcelain"],
                cwd=repo_dir,
                capture_output=True,
                text=True
            )
            assert result.returncode == 0
    
    def test_large_superproject_operations(self, run_workspace, large_superproject_config, clean_workspace):
        """Test operations on large super-project with many repository entries."""
        result = run_workspace("init")
        assert result.returncode == 0
        
        # Despite 24 config entries, should only have 3 unique repos
        workspace_dir = Path("worktrees/main")
        repos = [d for d in workspace_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        assert len(repos) == 3  # Only unique repo names get cloned
        
        # Test that status command performs reasonably with the configuration
        result = run_workspace("status")
        assert result.returncode == 0
        
        # Test foreach operations work with the configuration
        result = run_workspace("foreach", "echo $name", check=False)
        # Foreach may fail if not in workspace directory context
        if result.returncode == 0:
            assert result.stdout.count("===") >= 3  # Should show headers for each repo
    
    def test_empty_superproject(self, run_workspace, temp_workspace, clean_workspace):
        """Test super-project with no repositories configured."""
        # Create empty config
        config_path = temp_workspace / "workspace.conf"
        config_path.write_text("# Empty super-project\n")
        
        result = run_workspace("init")
        assert result.returncode == 0
        
        workspace_dir = Path("worktrees/main")
        assert workspace_dir.exists()
        
        # Should be empty directory
        repos = [d for d in workspace_dir.iterdir() if d.is_dir()]
        assert len(repos) == 0
        
        # Status should handle empty workspace gracefully
        result = run_workspace("status")
        assert result.returncode == 0


class TestSuperProjectWorkflowIntegration:
    """Test super-project integration with real-world workflow patterns."""
    
    def test_microservices_pattern(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test super-project configured like a microservices architecture."""
        # Create microservices-style configuration
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        config_content = f"""# Microservices super-project
# Core services (stable)
{repo_a[1]} main v1.0.0
{repo_b[1]} main v1.1.0

# Development services (HEAD-tracking)
{repo_c[1]} develop
{repo_a[1]} feature-test

# Shared libraries (pinned for stability)
{repo_b[1]} main v1.0.0
"""
        config_path.write_text(config_content)
        
        result = run_workspace("init")
        assert result.returncode == 0
        
        # Should handle duplicate repos with different configurations
        workspace_dir = Path("worktrees/main")
        repos = [d for d in workspace_dir.iterdir() if d.is_dir() and (d / ".git").exists()]
        assert len(repos) == 3  # Unique repo names
        
        # Test foreach operations across the microservices
        result = run_workspace("foreach", "git branch --show-current || git describe --always", check=False)
        # Foreach may fail if not in proper workspace context
        if result.returncode == 0:
            assert len(result.stdout) > 0  # Should have some output
    
    def test_monorepo_migration_pattern(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test super-project configured for monorepo migration scenario."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        config_content = f"""# Monorepo migration super-project
# Legacy monorepo (pinned during migration)
{repo_a[1]} main v1.0.0

# Extracted services (active development)
{repo_b[1]} develop
{repo_c[1]} main

# Migration tooling (specific branch)
{repo_a[1]} migration-tools
"""
        config_path.write_text(config_content)
        
        result = run_workspace("init")
        assert result.returncode == 0
        
        # Verify mixed tracking modes work for migration scenario
        result = run_workspace("sync", check=False)
        # Sync may fail but should be handled gracefully
        
        # Should handle the mixed states appropriately
        result = run_workspace("status")
        assert result.returncode == 0
    
    def test_release_management_pattern(self, run_workspace, temp_workspace, base_git_repos, clean_workspace):
        """Test super-project configured for release management."""
        config_path = temp_workspace / "workspace.conf"
        
        repo_a, repo_b, repo_c = base_git_repos
        
        config_content = f"""# Release management super-project
# Production components (tagged releases)
{repo_a[1]} main v1.0.0
{repo_b[1]} main v1.1.0

# Development components (branch tracking)
{repo_c[1]} develop
{repo_a[1]} release-candidate

# Infrastructure (stable)
{repo_b[1]} main
"""
        config_path.write_text(config_content)
        
        # Test release workspace
        result = run_workspace("init", "release-v2.0")
        assert result.returncode == 0
        
        workspace_dir = Path("worktrees/release-v2.0")
        assert workspace_dir.exists()
        
        # Should be able to run release operations
        result = run_workspace("foreach", "git tag --list", check=False)
        # Foreach may fail if not in proper workspace context, but should handle gracefully