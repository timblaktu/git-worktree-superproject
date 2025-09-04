"""Test config command error handling and edge cases."""

import os
import shutil
import tempfile
from pathlib import Path
import pytest
import subprocess


class TestConfigCommandErrors:
    """Test error handling for all config subcommands."""

    def test_config_help_command(self, temp_workspace_git_enabled, run_workspace):
        """Test config help subcommand."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test config help
        result = run_workspace("config", "help", check=False)
        assert result.returncode == 0
        assert "set <workspace>" in result.stdout
        assert "show" in result.stdout
        assert "import" in result.stdout
        assert "set-default" in result.stdout

    def test_config_invalid_subcommand(self, temp_workspace_git_enabled, run_workspace):
        """Test config with invalid subcommand."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test invalid subcommand
        result = run_workspace("config", "invalid", check=False)
        assert result.returncode != 0
        assert "Unknown config subcommand" in result.stderr or "Unknown config subcommand" in result.stdout

    def test_config_no_subcommand(self, temp_workspace_git_enabled, run_workspace):
        """Test config without subcommand."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test config without subcommand
        result = run_workspace("config", check=False)
        assert result.returncode != 0
        assert "requires a subcommand" in result.stderr or "requires a subcommand" in result.stdout

    def test_config_set_invalid_workspace(self, temp_workspace_git_enabled, run_workspace):
        """Test config set with invalid workspace names."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test with empty workspace name
        result = run_workspace("config", "set", "", "https://example.com/repo.git", check=False)
        assert result.returncode != 0
        
        # Test with workspace name containing invalid characters
        result = run_workspace("config", "set", "../bad", "https://example.com/repo.git", check=False)
        assert result.returncode != 0
        
        # Test with workspace name containing spaces
        result = run_workspace("config", "set", "my workspace", "https://example.com/repo.git", check=False)
        # This might be allowed, depends on implementation

    def test_config_set_malformed_url(self, temp_workspace_git_enabled, run_workspace):
        """Test config set with malformed URLs."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test with invalid URL
        result = run_workspace("config", "set", "test", "not a url", check=False)
        # Note: The tool might accept this as a local path
        
        # Test with empty URL
        result = run_workspace("config", "set", "test", "", check=False)
        assert result.returncode != 0
        
        # Test with URL containing special characters
        result = run_workspace("config", "set", "test", "https://example.com/repo with spaces.git", check=False)
        # Should handle spaces appropriately

    def test_config_set_invalid_ref(self, temp_workspace_git_enabled, run_workspace):
        """Test config set with invalid refs."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test with invalid ref format
        result = run_workspace("config", "set", "test", "https://example.com/repo.git", "", "..badref", check=False)
        # Git might allow this, but it's questionable
        
        # Test with empty ref (should be valid - means no pinning)
        result = run_workspace("config", "set", "test", "https://example.com/repo.git", "", "", check=False)
        assert result.returncode == 0

    def test_config_import_nonexistent_file(self, temp_workspace_git_enabled, run_workspace):
        """Test config import with non-existent file."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test import from non-existent file
        result = run_workspace("config", "import", "test", "/non/existent/file.conf", check=False)
        assert result.returncode != 0
        assert "not found" in result.stderr.lower() or "not found" in result.stdout.lower()

    def test_config_import_invalid_file(self, temp_workspace_git_enabled, run_workspace):
        """Test config import with invalid file content."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create invalid config file
        invalid_config = workspace_dir / "invalid.conf"
        invalid_config.write_text("this is not a valid config format\n")
        
        # Test import
        result = run_workspace("config", "import", "test", str(invalid_config), check=False)
        # Might succeed but with warnings, or might fail

    def test_config_import_empty_file(self, temp_workspace_git_enabled, run_workspace):
        """Test config import with empty file."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create empty config file
        empty_config = workspace_dir / "empty.conf"
        empty_config.write_text("")
        
        # Test import (should succeed but do nothing)
        result = run_workspace("config", "import", "test", str(empty_config), check=False)
        assert result.returncode == 0

    def test_config_show_nonexistent_workspace(self, temp_workspace_git_enabled, run_workspace):
        """Test config show for non-existent workspace."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test show for non-existent workspace
        result = run_workspace("config", "show", "nonexistent", check=False)
        # Should succeed but show empty or default config
        assert result.returncode == 0

    def test_config_show_corrupted_git_config(self, temp_workspace_git_enabled, run_workspace):
        """Test config show with corrupted git config."""
        workspace_dir = temp_workspace_git_enabled
        
        # Initialize superproject if not already
        if not (workspace_dir / ".git").exists():
            subprocess.run(["git", "init"], cwd=workspace_dir, check=True)
        
        # Corrupt git config
        git_config = workspace_dir / ".git" / "config"
        if git_config.exists():
            git_config.write_text("[invalid section without closing\n")
        
        # Test show (should handle corruption gracefully)
        result = run_workspace("config", "show", check=False)
        # Should either fail gracefully or show partial config

    def test_config_set_default_errors(self, temp_workspace_git_enabled, run_workspace):
        """Test config set-default error cases."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test with empty URL
        result = run_workspace("config", "set-default", "", check=False)
        assert result.returncode != 0
        
        # Test with too many arguments
        result = run_workspace("config", "set-default", "url", "branch", "ref", "extra", check=False)
        # Should either ignore extra args or error

    def test_config_operations_without_git(self, run_workspace):
        """Test config operations when git is not initialized."""
        # Create a completely fresh directory
        fresh_dir = Path(tempfile.mkdtemp())
        
        try:
            # Save current dir and change to fresh dir
            old_cwd = os.getcwd()
            os.chdir(fresh_dir)
            
            # Test config operations without git init
            result = run_workspace("config", "show", check=False)
            # Should initialize git or fail gracefully
            
            result = run_workspace("config", "set", "test", "https://example.com/repo.git", check=False)
            # Should initialize git or fail gracefully
            
        finally:
            os.chdir(old_cwd)
            shutil.rmtree(fresh_dir)


class TestConfigEdgeCases:
    """Test edge cases in configuration handling."""

    def test_config_with_special_characters(self, temp_workspace_git_enabled, run_workspace):
        """Test configuration with special characters in values."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test URL with special characters
        result = run_workspace(
            "config", "set", "test", "https://user:p@ss@example.com/repo.git",
            check=False
        )
        assert result.returncode == 0
        
        # Test branch name with special characters
        result = run_workspace(
            "config", "set", "test2", "https://example.com/repo.git", "feature/test-123",
            check=False
        )
        assert result.returncode == 0
        
        # Verify they were stored correctly
        show_result = run_workspace("config", "show", "test", check=False)
        assert "example.com" in show_result.stdout

    def test_config_with_very_long_values(self, temp_workspace_git_enabled, run_workspace):
        """Test configuration with very long values."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create very long URL
        long_url = "https://example.com/" + "a" * 1000 + "/repo.git"
        
        result = run_workspace("config", "set", "test", long_url, check=False)
        # Should handle long values appropriately
        
        # Create very long branch name
        long_branch = "feature/" + "b" * 200
        
        result = run_workspace("config", "set", "test2", "https://example.com/repo.git", long_branch, check=False)
        # Should handle long branch names

    def test_config_unicode_handling(self, temp_workspace_git_enabled, run_workspace):
        """Test configuration with Unicode characters."""
        workspace_dir = temp_workspace_git_enabled
        
        # Test with Unicode in repository path
        result = run_workspace(
            "config", "set", "test", "https://example.com/测试/repo.git",
            check=False
        )
        # Should handle Unicode appropriately
        
        # Test with Unicode in branch name
        result = run_workspace(
            "config", "set", "test2", "https://example.com/repo.git", "特性/测试",
            check=False
        )
        # Should handle Unicode branch names

    def test_config_concurrent_modifications(self, temp_workspace_git_enabled, run_workspace):
        """Test concurrent configuration modifications."""
        workspace_dir = temp_workspace_git_enabled
        
        import threading
        
        def modify_config(workspace_name, url):
            run_workspace("config", "set", workspace_name, url, check=False)
        
        # Start multiple threads modifying config
        threads = []
        for i in range(5):
            t = threading.Thread(
                target=modify_config,
                args=(f"workspace{i}", f"https://example.com/repo{i}.git")
            )
            threads.append(t)
            t.start()
        
        # Wait for all threads
        for t in threads:
            t.join()
        
        # Verify all configs were set
        for i in range(5):
            result = run_workspace("config", "show", f"workspace{i}", check=False)
            assert result.returncode == 0

    def test_config_inheritance_complex(self, temp_workspace_git_enabled, run_workspace):
        """Test complex configuration inheritance scenarios."""
        workspace_dir = temp_workspace_git_enabled
        
        # Set default config
        run_workspace("config", "set-default", "https://default.com/repo.git", "main", check=False)
        
        # Set workspace-specific partial override
        run_workspace("config", "set", "test", "https://override.com/repo.git", check=False)
        
        # Create workspace.conf for legacy config
        config_file = workspace_dir / "workspace.conf"
        config_file.write_text("repo https://legacy.com/legacy.git legacy-branch\n")
        
        # Test inheritance priority
        result = run_workspace("config", "show", "test", check=False)
        assert "override.com" in result.stdout or "https://override.com" in result.stdout
        
        # Test fallback to default
        result = run_workspace("config", "show", "other", check=False)
        # Should show default or legacy config


class TestConfigMigration:
    """Test configuration migration scenarios."""

    def test_migrate_from_workspace_conf(self, temp_workspace_git_enabled, run_workspace):
        """Test migrating from workspace.conf to git config."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create legacy workspace.conf
        config_file = workspace_dir / "workspace.conf"
        config_file.write_text("""
repo https://example.com/repo1.git main
repo https://example.com/repo2.git feature-branch
repo https://example.com/repo3.git pinned abc123
""")
        
        # Import to new config
        result = run_workspace("config", "import", "migrated", check=False)
        assert result.returncode == 0
        
        # Verify migration
        show_result = run_workspace("config", "show", "migrated", check=False)
        assert "repo1" in show_result.stdout
        assert "repo2" in show_result.stdout
        assert "repo3" in show_result.stdout

    def test_mixed_config_sources(self, temp_workspace_git_enabled, run_workspace):
        """Test handling mixed configuration sources."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create workspace.conf
        config_file = workspace_dir / "workspace.conf"
        config_file.write_text("repo https://legacy.com/repo.git\n")
        
        # Set default config
        run_workspace("config", "set-default", "https://default.com/repo.git", check=False)
        
        # Set workspace-specific config
        run_workspace("config", "set", "test", "https://specific.com/repo.git", check=False)
        
        # Create workspace and verify correct config is used
        result = run_workspace("switch", "test", check=False)
        # Should use workspace-specific config
        
        # Verify repo was cloned from correct source
        if result.returncode == 0:
            # Check the remote URL in the created workspace
            repo_path = workspace_dir / "worktrees" / "test" / "repo"
            if repo_path.exists():
                remote_result = subprocess.run(
                    ["git", "remote", "get-url", "origin"],
                    cwd=repo_path,
                    capture_output=True,
                    text=True
                )
                assert "specific.com" in remote_result.stdout

    def test_config_backwards_compatibility(self, temp_workspace_git_enabled, run_workspace):
        """Test backwards compatibility with old config formats."""
        workspace_dir = temp_workspace_git_enabled
        
        # Create old-style workspace.conf with various formats
        config_file = workspace_dir / "workspace.conf"
        config_file.write_text("""
# Comment line
repo https://example.com/repo1.git
repo https://example.com/repo2.git branch-name
repo https://example.com/repo3.git @pinned ref123

# Another comment
repo git@github.com:user/repo.git
repo ../local/path
repo file:///absolute/path/repo.git
""")
        
        # Import and verify all formats are handled
        result = run_workspace("config", "import", "compat", check=False)
        assert result.returncode == 0
        
        # Show imported config
        show_result = run_workspace("config", "show", "compat", check=False)
        assert result.returncode == 0