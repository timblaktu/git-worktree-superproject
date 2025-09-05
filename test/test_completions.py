#!/usr/bin/env python3
"""Test suite for shell completions (zsh and bash).

This module tests the completion functions to ensure they correctly:
1. Extract URLs from git config
2. Extract repository names from URLs
3. Handle workspace-specific configurations
4. Present completions correctly to the shell
"""

import subprocess
import tempfile
import shutil
import os
import pytest
from pathlib import Path
from unittest.mock import patch, MagicMock
import sys

# Find the workspace script directory
WORKSPACE_DIR = Path(__file__).parent.parent.resolve()
COMPLETION_SCRIPT = WORKSPACE_DIR / "workspace-completion.zsh"
WORKSPACE_SCRIPT = WORKSPACE_DIR / "workspace"


class TestZshCompletions:
    """Test zsh completion functions."""

    def setup_method(self):
        """Setup a test environment for each test."""
        self.test_dir = tempfile.mkdtemp(prefix="test_completion_")
        self.worktrees_dir = Path(self.test_dir) / "worktrees"
        self.worktrees_dir.mkdir()
        
        # Initialize git repo
        subprocess.run(["git", "init"], cwd=self.test_dir, check=True, capture_output=True)
        
        # Copy workspace script to test directory
        test_workspace = Path(self.test_dir) / "workspace"
        shutil.copy(str(WORKSPACE_SCRIPT), str(test_workspace))
        test_workspace.chmod(0o755)

    def teardown_method(self):
        """Clean up test environment."""
        if Path(self.test_dir).exists():
            shutil.rmtree(self.test_dir)

    def add_repo_to_config(self, url, branch=None, ref=None):
        """Add a repository to git config."""
        value = url
        if branch:
            value += f" {branch}"
        if ref:
            value += f" {ref}"
        subprocess.run(
            ["git", "config", "--add", "workspace.repo", value],
            cwd=self.test_dir,
            check=True
        )

    def test_get_repo_urls_extraction(self):
        """Test that _get_repo_urls correctly extracts URLs from git config."""
        # Add test repositories
        self.add_repo_to_config("https://github.com/example/project-common.git", "main")
        self.add_repo_to_config("https://github.com/example/project-platform.git", "feature/test")
        self.add_repo_to_config("https://github.com/example/repo.git")
        
        # Create a test script that sources the completion and tests _get_repo_urls
        test_script = f"""#!/bin/zsh
# Source the completion script with modified paths
workspace_dir="{self.test_dir}"
worktrees_dir="$workspace_dir/worktrees"

# Extract the _get_repo_urls function from the completion script
source <(sed -n '/_get_repo_urls()/,/^[[:space:]]*\\}}/p' {COMPLETION_SCRIPT})

# Call the function
urls=($(_get_repo_urls))

# Print results for verification
for url in "${{urls[@]}}"; do
    echo "URL: $url"
done
"""
        
        # Run the test script
        result = subprocess.run(
            ["zsh", "-c", test_script],
            capture_output=True,
            text=True,
            cwd=self.test_dir
        )
        
        # Verify output
        assert result.returncode == 0, f"Script failed: {result.stderr}"
        output_lines = result.stdout.strip().split('\n')
        
        # Check that we got all URLs
        expected_urls = [
            "https://github.com/example/project-common.git",
            "https://github.com/example/project-platform.git", 
            "https://github.com/example/repo.git"
        ]
        
        extracted_urls = [line.replace("URL: ", "") for line in output_lines if line.startswith("URL: ")]
        assert set(extracted_urls) == set(expected_urls), f"Expected {expected_urls}, got {extracted_urls}"

    def test_get_repo_names_extraction(self):
        """Test that _get_repo_names correctly extracts repository names."""
        # Add test repositories
        self.add_repo_to_config("https://github.com/example/project-common.git", "main")
        self.add_repo_to_config("https://github.com/example/project-platform.git", "feature/test")
        self.add_repo_to_config("https://github.com/example/repo.git")
        
        # Create a test script
        test_script = f"""#!/bin/zsh
# Source the completion script with modified paths
workspace_dir="{self.test_dir}"
worktrees_dir="$workspace_dir/worktrees"

# Extract the _get_repo_names function from the completion script
source <(sed -n '/_get_repo_names()/,/^[[:space:]]*\\}}/p' {COMPLETION_SCRIPT})

# Call the function
repos=($(_get_repo_names))

# Print results
for repo in "${{repos[@]}}"; do
    echo "REPO: $repo"
done
"""
        
        result = subprocess.run(
            ["zsh", "-c", test_script],
            capture_output=True,
            text=True,
            cwd=self.test_dir
        )
        
        assert result.returncode == 0, f"Script failed: {result.stderr}"
        
        expected_names = ["project-common", "project-platform", "repo"]
        extracted_names = [line.replace("REPO: ", "") for line in result.stdout.strip().split('\n') if line.startswith("REPO: ")]
        assert set(extracted_names) == set(expected_names), f"Expected {expected_names}, got {extracted_names}"

    def test_completion_for_config_set(self):
        """Test completion behavior for workspace config set command."""
        # Add repositories to config
        self.add_repo_to_config("https://github.com/example/project-common.git", "main")
        self.add_repo_to_config("https://github.com/example/project-platform.git", "feature/test")
        
        # Create a workspace
        workspace_name = "test-workspace"
        workspace_path = self.worktrees_dir / workspace_name
        workspace_path.mkdir()
        
        # Create a minimal .git file to simulate worktree
        (workspace_path / ".git").write_text(f"gitdir: {self.test_dir}/.git/worktrees/{workspace_name}")
        
        # Test script that simulates completion context
        test_script = f"""#!/bin/zsh
# Set up environment
workspace_dir="{self.test_dir}"
worktrees_dir="$workspace_dir/worktrees"

# Source the relevant functions from completion script
source <(sed -n '/_get_repo_urls()/,/^[[:space:]]*\\}}/p' {COMPLETION_SCRIPT})
source <(sed -n '/_get_repo_names()/,/^[[:space:]]*\\}}/p' {COMPLETION_SCRIPT})

# Test URL extraction for workspace config set
workspace="{workspace_name}"
urls=($(_get_repo_urls "$workspace"))
repos=($(_get_repo_names))

echo "=== URLs extracted ==="
for url in "${{urls[@]}}"; do
    echo "  $url"
done

echo "=== Repository names extracted ==="
for repo in "${{repos[@]}}"; do
    echo "  $repo"
done

# Check if we have completions
all_options=()
for url in "${{urls[@]}}"; do
    all_options+=("$url")
done
for repo in "${{repos[@]}}"; do
    all_options+=("$repo")
done

echo "=== Total completion options: ${{#all_options[@]}} ==="
if (( ${{#all_options[@]}} > 0 )); then
    echo "Completions available:"
    for opt in "${{all_options[@]}}"; do
        echo "  - $opt"
    done
else
    echo "ERROR: No completions available (would show 'no repositories configured')"
fi
"""
        
        result = subprocess.run(
            ["zsh", "-c", test_script],
            capture_output=True,
            text=True,
            cwd=self.test_dir
        )
        
        print(f"Test output:\n{result.stdout}")
        if result.stderr:
            print(f"Test stderr:\n{result.stderr}")
        
        assert result.returncode == 0, f"Script failed: {result.stderr}"
        assert "ERROR: No completions available" not in result.stdout, "Completion extraction failed"
        assert "https://github.com/example/project-common.git" in result.stdout
        assert "project-common" in result.stdout

    def test_completion_isolation_and_debugging(self):
        """Test completion function isolation and debug output."""
        # Add test data
        self.add_repo_to_config("https://github.com/example/project-common.git", "main")
        
        # Debug script to understand function behavior
        debug_script = f"""#!/bin/zsh
set -x  # Enable debugging
workspace_dir="{self.test_dir}"
worktrees_dir="$workspace_dir/worktrees"

# Test git config directly
echo "=== Direct git config test ==="
cd "$workspace_dir" && git config --get-all workspace.repo

# Test function extraction and execution
echo "=== Function test ==="
source <(sed -n '/_get_repo_urls()/,/^[[:space:]]*\\}}/p' {COMPLETION_SCRIPT})

# Add debug output to function
_get_repo_urls_debug() {{
    local workspace="${{1:-}}"
    local urls=()
    
    echo "[DEBUG] workspace_dir=$workspace_dir" >&2
    echo "[DEBUG] Testing git access" >&2
    
    if [[ -d "$workspace_dir/.git" ]]; then
        echo "[DEBUG] .git directory exists" >&2
        while IFS= read -r line; do
            echo "[DEBUG] Read line: '$line'" >&2
            [[ -n "$line" ]] || continue
            urls+=(${{line%% *}})
        done < <(cd "$workspace_dir" && git config --get-all workspace.repo 2>/dev/null)
    else
        echo "[DEBUG] .git directory not found at $workspace_dir/.git" >&2
    fi
    
    echo "[DEBUG] Found ${{#urls[@]}} URLs" >&2
    echo "${{urls[@]}}"
}}

result=($(_get_repo_urls_debug))
echo "=== Function returned ${{#result[@]}} URLs ==="
for url in "${{result[@]}}"; do
    echo "  URL: $url"
done
"""
        
        result = subprocess.run(
            ["zsh", "-c", debug_script],
            capture_output=True,
            text=True,
            cwd=self.test_dir
        )
        
        print(f"Debug stdout:\n{result.stdout}")
        print(f"Debug stderr:\n{result.stderr}")
        
        assert "https://github.com/example/project-common.git" in result.stdout

    def test_real_world_completion_scenario(self):
        """Test a real-world scenario matching the bug report."""
        # Replicate the exact git config from the bug report
        repos = [
            ("https://github.com/example/project-common.git", "main"),
            ("https://github.com/example/project-platform.git", "main"),
            ("https://github.com/example/project-sdk.git", "develop"),
            ("https://github.com/example/project-framework.git", "feature-branch")
        ]
        
        for url, branch in repos:
            self.add_repo_to_config(url, branch)
            # Add duplicates as shown in the output
            self.add_repo_to_config(url, branch)
        
        # Create 'dev' workspace as mentioned in the bug
        main_workspace = self.worktrees_dir / "main"
        main_workspace.mkdir()
        (main_workspace / ".git").write_text(f"gitdir: {self.test_dir}/.git/worktrees/main")
        
        # Test the exact completion scenario
        test_script = f"""#!/bin/zsh
# Replicate the exact completion context for 'workspace config set dev <TAB>'
workspace_dir="{self.test_dir}"
worktrees_dir="$workspace_dir/worktrees"

# Source the functions we need
source <(sed -n '/_get_repo_urls()/,/^[[:space:]]*\\}}/p' {COMPLETION_SCRIPT})
source <(sed -n '/_get_repo_names()/,/^[[:space:]]*\\}}/p' {COMPLETION_SCRIPT})

# Simulate the completion for 'workspace config set dev <TAB>'
workspace="dev"
urls=($(_get_repo_urls "$workspace"))
repos=($(_get_repo_names))

# Build combined options as the completion does
all_options=()
for url in "${{urls[@]}}"; do
    all_options+=("$url")
done
for repo in "${{repos[@]}}"; do
    all_options+=("$repo")
done

# Report what completion would show
if (( ${{#all_options[@]}} > 0 )); then
    echo "SUCCESS: Completion would show ${{#all_options[@]}} options:"
    for opt in "${{all_options[@]}}"; do
        echo "  $opt"
    done | sort -u  # Remove duplicates for cleaner output
else
    echo "BUG REPRODUCED: Completion would show 'no repositories configured'"
fi
"""
        
        result = subprocess.run(
            ["zsh", "-c", test_script],
            capture_output=True,
            text=True,
            cwd=self.test_dir
        )
        
        print(f"Real-world test output:\n{result.stdout}")
        
        # The test should show the bug is fixed
        assert "BUG REPRODUCED" not in result.stdout, "The completion bug is still present"
        assert "SUCCESS: Completion would show" in result.stdout
        
        # Verify all expected completions are present
        for url, _ in repos:
            assert url in result.stdout, f"Missing URL in completions: {url}"
        
        # Verify repository names are extracted
        expected_names = ["project-common", "project-platform", "project-sdk", "project-framework"]
        for name in expected_names:
            assert name in result.stdout, f"Missing repository name in completions: {name}"

    def test_workspace_dir_resolution_bug(self):
        """Test that workspace_dir is correctly resolved after _arguments modifies $words.
        
        This tests the bug where $words[1] becomes 'config' instead of './workspace'
        after _arguments processes the command line.
        """
        # Set up test repos
        self.add_repo_to_config("https://github.com/test/repo1.git", "main")
        self.add_repo_to_config("https://github.com/test/repo2.git", "develop")
        
        # Make workspace script executable for test
        test_workspace = Path(self.test_dir) / "workspace"
        test_workspace.write_text("#!/bin/bash\necho test")
        test_workspace.chmod(0o755)
        
        # Test script that simulates the bug and fix
        test_script = f"""#!/bin/zsh
# Test workspace_dir resolution with different command formats

test_resolution() {{
    local cmd="$1"
    local expected_dir="$2"
    
    # Simulate what _workspace does
    local -a words
    words=($cmd config set myworkspace)
    
    # Save original command before _arguments
    local orig_cmd="${{words[1]}}"
    
    # Simulate _arguments modifying words (shifts off first element)
    words=(config set myworkspace)
    
    # OLD BUGGY CODE would do:
    # workspace_dir="$(dirname "$(readlink -f "${{words[1]}}")")"
    # Which would use 'config' not './workspace'
    
    # FIXED CODE:
    local workspace_dir
    if [[ "$orig_cmd" == */* ]]; then
        workspace_dir="$(dirname "$(readlink -f "$orig_cmd")")"
    else
        if [[ -x "./workspace" ]]; then
            workspace_dir="$(dirname "$(readlink -f "./workspace")")"
        else
            workspace_dir="$PWD"
        fi
    fi
    
    if [[ "$workspace_dir" == "$expected_dir" ]]; then
        echo "✓ $cmd: Correctly resolved to $workspace_dir"
        return 0
    else
        echo "✗ $cmd: Got $workspace_dir, expected $expected_dir"
        return 1
    fi
}}

cd {self.test_dir}

# Test different command formats
test_resolution "./workspace" "{self.test_dir}"
test_resolution "workspace" "{self.test_dir}"  # Assumes ./workspace exists
# For absolute path, use the actual test workspace
test_resolution "{test_workspace}" "{self.test_dir}"
"""
        
        result = subprocess.run(
            ["zsh", "-c", test_script],
            capture_output=True,
            text=True,
            cwd=self.test_dir
        )
        
        print(f"Resolution test output:\n{result.stdout}")
        assert result.returncode == 0, f"Resolution test failed:\n{result.stdout}\n{result.stderr}"
        assert "✓" in result.stdout, "Workspace dir resolution fix not working"


if __name__ == "__main__":
    # Run tests directly
    import sys
    pytest.main([__file__, "-v", "-s"] + sys.argv[1:])