"""Pytest fixtures for testing the workspace management tool."""

import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Generator, List, Tuple

import pytest


# pytest-result-bar plugin will auto-register via entry points
# No need to manually specify pytest_plugins anymore


@pytest.fixture
def temp_workspace(tmp_path: Path) -> Generator[Path, None, None]:
    """Create a temporary workspace directory for testing."""
    workspace_dir = tmp_path / "test_workspace"
    workspace_dir.mkdir(exist_ok=True)
    
    # Save current directory
    original_dir = os.getcwd()
    
    try:
        os.chdir(workspace_dir)
        yield workspace_dir
    finally:
        # Restore original directory
        os.chdir(original_dir)


# Base git repositories - module scoped for shared read-only access
@pytest.fixture(scope="module")
def base_git_repos(tmp_path_factory) -> List[Tuple[str, Path]]:
    """Create base git repositories shared across tests in a module.
    
    These repositories should NOT be modified by tests.
    Returns a list of (repo_name, repo_path) tuples.
    """
    repos_dir = tmp_path_factory.mktemp("base_repos")
    
    repos = []
    
    # Create test repositories
    for repo_name in ["repo-a", "repo-b", "repo-c"]:
        repo_path = repos_dir / repo_name
        repo_path.mkdir()
        
        # Initialize git repo
        subprocess.run(["git", "init"], cwd=repo_path, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], 
                      cwd=repo_path, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "Test User"], 
                      cwd=repo_path, check=True, capture_output=True)
        
        # Create initial commit
        (repo_path / "README.md").write_text(f"# {repo_name}\nTest repository")
        subprocess.run(["git", "add", "."], cwd=repo_path, check=True, capture_output=True)
        subprocess.run(["git", "commit", "-m", "Initial commit"], 
                      cwd=repo_path, check=True, capture_output=True)
        
        # Ensure we have a main branch
        subprocess.run(["git", "branch", "-M", "main"], cwd=repo_path, check=True, capture_output=True)
        
        # Create some branches
        for branch in ["develop", "feature-test"]:
            subprocess.run(["git", "checkout", "-b", branch], 
                          cwd=repo_path, check=True, capture_output=True)
            (repo_path / f"{branch}.txt").write_text(f"Content for {branch}")
            subprocess.run(["git", "add", "."], cwd=repo_path, check=True, capture_output=True)
            subprocess.run(["git", "commit", "-m", f"Add {branch} file"], 
                          cwd=repo_path, check=True, capture_output=True)
        
        # Create a tag on main branch
        subprocess.run(["git", "checkout", "main"], cwd=repo_path, check=True, capture_output=True)
        subprocess.run(["git", "tag", "v1.0.0"], cwd=repo_path, check=True, capture_output=True)
        
        # Create additional commits for testing commit SHAs
        for i in range(1, 4):
            (repo_path / f"commit-{i}.txt").write_text(f"Commit {i} content")
            subprocess.run(["git", "add", "."], cwd=repo_path, check=True, capture_output=True)
            subprocess.run(["git", "commit", "-m", f"Commit {i}"], 
                          cwd=repo_path, check=True, capture_output=True)
            if i == 2:
                # Create a second tag
                subprocess.run(["git", "tag", "v1.1.0"], cwd=repo_path, check=True, capture_output=True)
        
        repos.append((repo_name, repo_path))
    
    return repos


@pytest.fixture
def git_repos(tmp_path: Path) -> Generator[List[Tuple[str, Path]], None, None]:
    """Create modifiable git repositories for testing.
    
    Returns a list of (repo_name, repo_path) tuples.
    Use this for tests that need to modify repositories.
    """
    repos_dir = tmp_path / "repos"
    repos_dir.mkdir()
    
    repos = []
    
    # Create test repositories
    for repo_name in ["repo-a", "repo-b", "repo-c"]:
        repo_path = repos_dir / repo_name
        repo_path.mkdir()
        
        # Initialize git repo
        subprocess.run(["git", "init"], cwd=repo_path, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], 
                      cwd=repo_path, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "Test User"], 
                      cwd=repo_path, check=True, capture_output=True)
        
        # Create initial commit
        (repo_path / "README.md").write_text(f"# {repo_name}\nTest repository")
        subprocess.run(["git", "add", "."], cwd=repo_path, check=True, capture_output=True)
        subprocess.run(["git", "commit", "-m", "Initial commit"], 
                      cwd=repo_path, check=True, capture_output=True)
        
        # Ensure we have a main branch
        subprocess.run(["git", "branch", "-M", "main"], cwd=repo_path, check=True, capture_output=True)
        
        # Create some branches
        for branch in ["develop", "feature-test"]:
            subprocess.run(["git", "checkout", "-b", branch], 
                          cwd=repo_path, check=True, capture_output=True)
            (repo_path / f"{branch}.txt").write_text(f"Content for {branch}")
            subprocess.run(["git", "add", "."], cwd=repo_path, check=True, capture_output=True)
            subprocess.run(["git", "commit", "-m", f"Add {branch} file"], 
                          cwd=repo_path, check=True, capture_output=True)
        
        # Create a tag on main branch
        subprocess.run(["git", "checkout", "main"], cwd=repo_path, check=True, capture_output=True)
        subprocess.run(["git", "tag", "v1.0.0"], cwd=repo_path, check=True, capture_output=True)
        
        repos.append((repo_name, repo_path))
    
    yield repos


@pytest.fixture
def workspace_script(temp_workspace: Path) -> Path:
    """Copy the workspace script to the test directory."""
    script_source = Path(__file__).parent.parent / "workspace"
    script_dest = temp_workspace / "workspace"
    
    shutil.copy2(script_source, script_dest)
    script_dest.chmod(0o755)
    
    return script_dest


@pytest.fixture
def workspace_config(temp_workspace: Path, git_repos: List[Tuple[str, Path]]) -> Path:
    """Create a workspace configuration file."""
    config_path = temp_workspace / "workspace.conf"
    
    config_lines = ["# Test workspace configuration"]
    for repo_name, repo_path in git_repos:
        config_lines.append(f"{repo_path}")
    config_lines.append("")  # Ensure trailing newline
    
    config_path.write_text("\n".join(config_lines))
    return config_path


@pytest.fixture
def run_workspace(workspace_script: Path):
    """Helper function to run workspace commands."""
    def _run(*args, check=True, capture_output=True, text=True, input=None, env=None):
        """Run workspace command with given arguments."""
        cmd = [str(workspace_script)] + list(args)
        
        # Ensure git config is inherited from environment
        run_env = os.environ.copy()
        if env:
            run_env.update(env)
            
        result = subprocess.run(
            cmd,
            check=check,
            capture_output=capture_output,
            text=text,
            input=input,
            env=run_env,
            cwd=os.getcwd()  # Run from current test directory
        )
        return result
    return _run


@pytest.fixture
def clean_workspace(temp_workspace: Path):
    """Clean up worktrees directory after each test."""
    yield
    worktrees_dir = temp_workspace / "worktrees"
    if worktrees_dir.exists():
        shutil.rmtree(worktrees_dir)


# Class-scoped fixtures for read-only tests
@pytest.fixture(scope="class")
def readonly_workspace_config(request, base_git_repos: List[Tuple[str, Path]]) -> Path:
    """Create a workspace configuration for read-only tests (class-scoped)."""
    # Use tmp_path_factory to create temp dir that persists for the class
    tmp_factory = request.config._tmp_path_factory
    temp_workspace = tmp_factory.mktemp("readonly_workspace")
    
    # Change to temp workspace directory
    original_dir = os.getcwd()
    os.chdir(temp_workspace)
    
    def cleanup():
        os.chdir(original_dir)
    request.addfinalizer(cleanup)
    
    config_path = temp_workspace / "workspace.conf"
    
    config_lines = ["# Read-only test workspace configuration"]
    for repo_name, repo_path in base_git_repos:
        config_lines.append(f"{repo_path}")
    config_lines.append("")  # Ensure trailing newline
    
    config_path.write_text("\n".join(config_lines))
    return config_path


@pytest.fixture
def complex_workspace_config(temp_workspace: Path, git_repos: List[Tuple[str, Path]]) -> Path:
    """Create a complex workspace configuration with various scenarios."""
    config_path = temp_workspace / "workspace.conf"
    
    repo_a, repo_b, repo_c = git_repos
    
    config_content = f"""# Complex workspace configuration
# Standard repositories
{repo_a[1]}
{repo_b[1]} develop

# Pinned repository
{repo_c[1]} main v1.0.0

# Comments and empty lines should be ignored

# This is another comment
"""
    
    config_path.write_text(config_content)
    return config_path


# Super-project configuration fixtures for heterogeneous testing
@pytest.fixture
def minimal_superproject_config(temp_workspace: Path, base_git_repos: List[Tuple[str, Path]]) -> Path:
    """Create minimal super-project with 1-2 repos for testing basic scenarios."""
    config_path = temp_workspace / "workspace.conf"
    
    repo_a, repo_b, _ = base_git_repos
    config_content = f"""# Minimal super-project
{repo_a[1]}
{repo_b[1]} develop
"""
    
    config_path.write_text(config_content)
    return config_path


@pytest.fixture 
def heterogeneous_superproject_config(temp_workspace: Path, base_git_repos: List[Tuple[str, Path]]) -> Path:
    """Create heterogeneous super-project with mixed tracking modes."""
    config_path = temp_workspace / "workspace.conf"
    
    repo_a, repo_b, repo_c = base_git_repos
    
    # Get actual commit SHAs for testing
    short_sha = subprocess.run(
        ["git", "rev-parse", "--short", "HEAD"], 
        cwd=repo_c[1], capture_output=True, text=True, check=True
    ).stdout.strip()
    
    config_content = f"""# Heterogeneous super-project configuration
# HEAD-tracking repos (follow branch)
{repo_a[1]}
{repo_b[1]} develop

# Detached repos (pinned)
{repo_c[1]} main v1.0.0
{repo_a[1]} feature-test
{repo_b[1]} main {short_sha}
"""
    
    config_path.write_text(config_content)
    return config_path


@pytest.fixture
def large_superproject_config(temp_workspace: Path, base_git_repos: List[Tuple[str, Path]]) -> Path:
    """Create large super-project with many repositories for scale testing."""
    config_path = temp_workspace / "workspace.conf"
    
    repo_a, repo_b, repo_c = base_git_repos
    
    config_lines = ["# Large super-project configuration"]
    
    # Add repos with different configurations multiple times to simulate scale
    branches = ["main", "develop", "feature-test"]
    refs = ["", "v1.0.0", "v1.1.0", ""]
    
    for i in range(8):  # Create 24 total repo entries (8 * 3 repos)
        for j, (repo_name, repo_path) in enumerate(base_git_repos):
            branch = branches[j % len(branches)]  
            ref = refs[i % len(refs)]
            
            if ref:
                config_lines.append(f"{repo_path} {branch} {ref}")
            else:
                config_lines.append(f"{repo_path} {branch}")
    
    config_lines.append("")
    config_path.write_text("\n".join(config_lines))
    return config_path


@pytest.fixture
def temp_workspace_git_enabled(tmp_path: Path) -> Generator[Path, None, None]:
    """Create temp workspace with git initialization for per-workspace config tests."""
    workspace_dir = tmp_path / "test_workspace"
    workspace_dir.mkdir(exist_ok=True)
    
    # Save current directory
    original_dir = os.getcwd()
    
    try:
        os.chdir(workspace_dir)
        
        # Initialize as git repo
        subprocess.run(["git", "init"], check=True, capture_output=True)
        subprocess.run(["git", "config", "user.email", "test@example.com"], check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "Test User"], check=True, capture_output=True)
        subprocess.run(["git", "config", "extensions.worktreeConfig", "true"], check=True, capture_output=True)
        
        # Create initial commit
        (workspace_dir / ".gitignore").write_text("worktrees/\nrepos/\n")
        subprocess.run(["git", "add", "."], check=True, capture_output=True)
        subprocess.run(["git", "commit", "-m", "Initial"], check=True, capture_output=True)
        
        yield workspace_dir
    finally:
        # Restore original directory
        os.chdir(original_dir)


@pytest.fixture
def reference_types_config(temp_workspace: Path, base_git_repos: List[Tuple[str, Path]]) -> Path:
    """Create super-project config testing different reference types."""
    config_path = temp_workspace / "workspace.conf"
    
    repo_a, repo_b, repo_c = base_git_repos
    
    # Get various reference types from the repos
    full_sha = subprocess.run(
        ["git", "rev-parse", "HEAD"], 
        cwd=repo_a[1], capture_output=True, text=True, check=True
    ).stdout.strip()
    
    short_sha = subprocess.run(
        ["git", "rev-parse", "--short", "HEAD"], 
        cwd=repo_b[1], capture_output=True, text=True, check=True
    ).stdout.strip()
    
    config_content = f"""# Reference types super-project
# Branch HEAD tracking
{repo_a[1]} main
{repo_b[1]} develop

# Tag references  
{repo_c[1]} main v1.0.0
{repo_a[1]} main v1.1.0

# Commit SHA references
{repo_b[1]} main {full_sha}
{repo_c[1]} feature-test {short_sha}

# Branch + commit combinations
{repo_a[1]} develop {short_sha}
"""
    
    config_path.write_text(config_content)
    return config_path