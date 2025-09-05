#!/usr/bin/env python3
"""Test cases for handling broken/corrupted repository states"""

import os
import shutil
import subprocess
import pytest
from pathlib import Path
import tempfile


class TestBrokenRepositories:
    """Test handling of broken repository states"""
    
    def test_uninitialized_repository(self, temp_workspace):
        """Test handling of repository with no commits"""
        workspace_dir = temp_workspace
        
        # Create workspace.conf
        with open(os.path.join(workspace_dir, "workspace.conf"), "w") as f:
            f.write("https://github.com/test/repo.git main\n")
        
        # Create a mock repository with no commits
        repo_dir = os.path.join(workspace_dir, "worktrees", "main", "repo")
        os.makedirs(repo_dir, exist_ok=True)
        subprocess.run(["git", "init"], cwd=repo_dir, check=True)
        subprocess.run(["git", "remote", "add", "origin", "https://github.com/test/repo.git"], 
                      cwd=repo_dir, check=True)
        
        # Set HEAD to invalid branch
        head_file = os.path.join(repo_dir, ".git", "HEAD")
        with open(head_file, "w") as f:
            f.write("ref: refs/heads/.invalid\n")
        
        # Run status command - should handle gracefully
        result = subprocess.run(
            ["./workspace", "status"],
            cwd=workspace_dir,
            capture_output=True,
            text=True
        )
        
        assert "[uninitialized]" in result.stdout or "[no commits]" in result.stdout
        assert result.returncode == 0
    
    def test_corrupted_worktree(self, temp_workspace):
        """Test handling of corrupted worktree"""
        workspace_dir = temp_workspace
        
        # Create workspace.conf
        with open(os.path.join(workspace_dir, "workspace.conf"), "w") as f:
            f.write("https://github.com/test/repo.git main\n")
        
        # Create central repo
        central_dir = os.path.join(workspace_dir, "repos", "repo")
        os.makedirs(central_dir, exist_ok=True)
        subprocess.run(["git", "init"], cwd=central_dir, check=True)
        subprocess.run(["git", "remote", "add", "origin", "https://github.com/test/repo.git"], 
                      cwd=central_dir, check=True)
        
        # Create initial commit in central repo
        subprocess.run(["git", "commit", "--allow-empty", "-m", "Initial"], cwd=central_dir, check=True)
        
        # Create corrupted worktree
        worktree_dir = os.path.join(workspace_dir, "worktrees", "main", "repo")
        os.makedirs(worktree_dir, exist_ok=True)
        
        # Create invalid .git file
        with open(os.path.join(worktree_dir, ".git"), "w") as f:
            f.write("gitdir: /nonexistent/path\n")
        
        # Run status - should detect corruption
        result = subprocess.run(
            ["./workspace", "status"],
            cwd=workspace_dir,
            capture_output=True,
            text=True
        )
        
        assert "[broken]" in result.stdout or "[invalid" in result.stdout.lower()
    
    def test_repair_uninitialized_repo(self, temp_workspace):
        """Test repair command on uninitialized repository"""
        workspace_dir = temp_workspace
        
        # Create workspace.conf
        with open(os.path.join(workspace_dir, "workspace.conf"), "w") as f:
            f.write("https://github.com/test/repo.git main\n")
        
        # Create uninitialized repo
        repo_dir = os.path.join(workspace_dir, "worktrees", "main", "repo")
        os.makedirs(repo_dir, exist_ok=True)
        subprocess.run(["git", "init"], cwd=repo_dir, check=True)
        subprocess.run(["git", "remote", "add", "origin", "https://github.com/test/repo.git"], 
                      cwd=repo_dir, check=True)
        
        # Run repair command
        result = subprocess.run(
            ["./workspace", "repair", "main", "repo"],
            cwd=workspace_dir,
            capture_output=True,
            text=True,
            timeout=10  # Prevent hanging
        )
        
        assert "Attempting to repair" in result.stdout
    
    def test_standalone_repo_conversion(self, temp_workspace):
        """Test conversion of standalone repo to worktree"""
        workspace_dir = temp_workspace
        
        # Create workspace.conf
        with open(os.path.join(workspace_dir, "workspace.conf"), "w") as f:
            f.write("https://github.com/test/repo.git main\n")
        
        # Create standalone repo instead of worktree
        repo_dir = os.path.join(workspace_dir, "worktrees", "main", "repo")
        os.makedirs(os.path.dirname(repo_dir), exist_ok=True)
        subprocess.run(["git", "init", repo_dir], check=True)
        subprocess.run(["git", "remote", "add", "origin", "https://github.com/test/repo.git"], 
                      cwd=repo_dir, check=True)
        
        # Create a commit so it's not empty
        subprocess.run(["git", "commit", "--allow-empty", "-m", "Test"], cwd=repo_dir, check=True)
        
        # Run repair - should detect and offer to convert
        result = subprocess.run(
            ["./workspace", "repair", "main", "repo"],
            cwd=workspace_dir,
            capture_output=True,
            text=True,
            timeout=10
        )
        
        assert "standalone repository" in result.stdout.lower() or "worktree" in result.stdout.lower()
    
    def test_missing_central_repo(self, temp_workspace):
        """Test handling when central repo is missing"""
        workspace_dir = temp_workspace
        
        # Create workspace.conf
        with open(os.path.join(workspace_dir, "workspace.conf"), "w") as f:
            f.write("https://github.com/test/repo.git main\n")
        
        # Create worktree without central repo
        worktree_dir = os.path.join(workspace_dir, "worktrees", "main", "repo")
        os.makedirs(worktree_dir, exist_ok=True)
        
        # Create .git file pointing to missing central repo
        with open(os.path.join(worktree_dir, ".git"), "w") as f:
            f.write(f"gitdir: {workspace_dir}/repos/repo/.git/worktrees/main\n")
        
        # Run status - should detect missing central repo
        result = subprocess.run(
            ["./workspace", "status"],
            cwd=workspace_dir,
            capture_output=True,
            text=True
        )
        
        assert "[invalid" in result.stdout.lower() or "[broken]" in result.stdout
    
    def test_detached_head_handling(self, temp_workspace):
        """Test handling of detached HEAD state"""
        workspace_dir = temp_workspace
        
        # Create workspace.conf  
        with open(os.path.join(workspace_dir, "workspace.conf"), "w") as f:
            f.write("https://github.com/test/repo.git main\n")
        
        # Create proper repo with commits
        central_dir = os.path.join(workspace_dir, "repos", "repo")
        os.makedirs(central_dir, exist_ok=True)
        subprocess.run(["git", "init"], cwd=central_dir, check=True)
        subprocess.run(["git", "commit", "--allow-empty", "-m", "Initial"], cwd=central_dir, check=True)
        
        # Create worktree
        worktree_dir = os.path.join(workspace_dir, "worktrees", "main", "repo")
        subprocess.run(["git", "worktree", "add", worktree_dir, "main"], cwd=central_dir, check=True)
        
        # Detach HEAD
        subprocess.run(["git", "checkout", "HEAD^0"], cwd=worktree_dir, check=True)
        
        # Run status - should handle detached HEAD
        result = subprocess.run(
            ["./workspace", "status"],
            cwd=workspace_dir,
            capture_output=True,
            text=True
        )
        
        assert "[detached]" in result.stdout.lower() or result.returncode == 0