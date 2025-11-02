use crate::error::{Result, WorkspaceError};
use crate::fs::FileSystem;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration for the workspace manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Base directory for worktrees
    pub worktree_base: PathBuf,
    /// Main repository path
    pub main_repo: PathBuf,
    /// Default branch name
    pub default_branch: String,
    /// Enable Nix flake integration
    pub enable_nix: bool,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| WorkspaceError::ConfigError(format!("Failed to read config: {}", e)))?;

        let mut config: Self = toml::from_str(&contents)
            .map_err(|e| WorkspaceError::ConfigError(format!("Failed to parse config: {}", e)))?;

        // Expand tilde in paths
        config.worktree_base = FileSystem::expand_tilde(&config.worktree_base);
        config.main_repo = FileSystem::expand_tilde(&config.main_repo);

        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string_pretty(self).map_err(|e| {
            WorkspaceError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path, contents)
            .map_err(|e| WorkspaceError::ConfigError(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Create default configuration
    pub fn default_config() -> Self {
        Self {
            worktree_base: FileSystem::expand_tilde(&PathBuf::from("~/.worktrees")),
            main_repo: PathBuf::from("."),
            default_branch: "main".to_string(),
            enable_nix: true,
        }
    }

    /// Detect configuration from environment
    pub fn detect() -> Result<Self> {
        // Try to find config in standard locations
        let config_paths = [
            PathBuf::from(".workspace.toml"),
            dirs::home_dir()
                .unwrap_or_default()
                .join(".config/workspace/config.toml"),
        ];

        for path in &config_paths {
            if path.exists() {
                return Self::load(path);
            }
        }

        // Return default if no config found
        Ok(Self::default_config())
    }
}

/// Repository configuration entry from workspace.conf
#[derive(Debug, Clone, PartialEq)]
pub struct RepoConfig {
    /// Repository URL
    pub url: String,
    /// Branch name (defaults to "main" if not specified)
    pub branch: String,
    /// Optional ref (tag or commit)
    pub git_ref: Option<String>,
}

impl RepoConfig {
    /// Parse a single line from workspace.conf
    /// Format: <url> [branch] [ref]
    pub fn parse_line(line: &str) -> Option<Self> {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let url = parts[0].to_string();
        let branch = parts
            .get(1)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "main".to_string());
        let git_ref = parts.get(2).map(|s| s.to_string());

        Some(RepoConfig {
            url,
            branch,
            git_ref,
        })
    }

    /// Parse entire workspace.conf file
    pub fn parse_workspace_conf(content: &str) -> Vec<Self> {
        content.lines().filter_map(Self::parse_line).collect()
    }

    /// Format as git config value (for storing in workspace.repo)
    pub fn to_config_value(&self) -> String {
        let mut value = format!("{} {}", self.url, self.branch);
        if let Some(ref git_ref) = self.git_ref {
            value.push(' ');
            value.push_str(git_ref);
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ========================================================================
    // TOML Config Tests
    // ========================================================================

    #[test]
    fn test_config_round_trip() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let config = Config::default_config();
        config.save(&config_path).unwrap();

        let loaded = Config::load(&config_path).unwrap();
        assert_eq!(loaded.default_branch, config.default_branch);
        assert_eq!(loaded.enable_nix, config.enable_nix);
    }

    // ========================================================================
    // workspace.conf Parsing Tests (migrated from Python test_config.py)
    // ========================================================================

    #[test]
    fn test_simple_config() {
        // Test parsing simple configuration with just URLs
        let config_content = "\
# Simple configuration
https://github.com/user/repo-a
https://github.com/user/repo-b
";
        let repos = RepoConfig::parse_workspace_conf(config_content);

        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].url, "https://github.com/user/repo-a");
        assert_eq!(repos[0].branch, "main"); // default branch
        assert_eq!(repos[0].git_ref, None);

        assert_eq!(repos[1].url, "https://github.com/user/repo-b");
        assert_eq!(repos[1].branch, "main");
        assert_eq!(repos[1].git_ref, None);
    }

    #[test]
    fn test_config_with_branches() {
        // Test configuration with specific branches
        let config_content = "\
https://github.com/user/repo-a
https://github.com/user/repo-b develop
https://github.com/user/repo-c feature-test
";
        let repos = RepoConfig::parse_workspace_conf(config_content);

        assert_eq!(repos.len(), 3);

        // repo-a should use default branch (main)
        assert_eq!(repos[0].url, "https://github.com/user/repo-a");
        assert_eq!(repos[0].branch, "main");

        // repo-b should be on develop
        assert_eq!(repos[1].url, "https://github.com/user/repo-b");
        assert_eq!(repos[1].branch, "develop");

        // repo-c should be on feature-test
        assert_eq!(repos[2].url, "https://github.com/user/repo-c");
        assert_eq!(repos[2].branch, "feature-test");
    }

    #[test]
    fn test_config_with_refs() {
        // Test configuration with specific refs (tags/commits)
        let config_content = "\
https://github.com/user/repo-a
https://github.com/user/repo-b main
https://github.com/user/repo-c main v1.0.0
";
        let repos = RepoConfig::parse_workspace_conf(config_content);

        assert_eq!(repos.len(), 3);

        // repo-a: no ref specified
        assert_eq!(repos[0].url, "https://github.com/user/repo-a");
        assert_eq!(repos[0].branch, "main");
        assert_eq!(repos[0].git_ref, None);

        // repo-b: branch but no ref
        assert_eq!(repos[1].url, "https://github.com/user/repo-b");
        assert_eq!(repos[1].branch, "main");
        assert_eq!(repos[1].git_ref, None);

        // repo-c: branch AND ref
        assert_eq!(repos[2].url, "https://github.com/user/repo-c");
        assert_eq!(repos[2].branch, "main");
        assert_eq!(repos[2].git_ref, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_config_comments_and_empty_lines() {
        // Test that comments and empty lines are ignored
        let config_content = "\
# This is a comment

https://github.com/user/repo-a
# Another comment
   # Indented comment

https://github.com/user/repo-b

https://github.com/user/repo-c
";
        let repos = RepoConfig::parse_workspace_conf(config_content);

        // Should parse exactly 3 repos, ignoring all comments and empty lines
        assert_eq!(repos.len(), 3);
        assert_eq!(repos[0].url, "https://github.com/user/repo-a");
        assert_eq!(repos[1].url, "https://github.com/user/repo-b");
        assert_eq!(repos[2].url, "https://github.com/user/repo-c");
    }

    #[test]
    fn test_empty_configuration() {
        // Empty configuration file
        let config_content = "";
        let repos = RepoConfig::parse_workspace_conf(config_content);
        assert_eq!(repos.len(), 0);
    }

    #[test]
    fn test_only_newlines() {
        // Configuration with only newlines
        let config_content = "\n\n\n";
        let repos = RepoConfig::parse_workspace_conf(config_content);
        assert_eq!(repos.len(), 0);
    }

    #[test]
    fn test_only_comments() {
        // Configuration with only comments
        let config_content = "\
# Just comments here
# No actual repos

# More comments
";
        let repos = RepoConfig::parse_workspace_conf(config_content);
        assert_eq!(repos.len(), 0);
    }

    #[test]
    fn test_only_whitespace() {
        // Configuration with only whitespace
        let config_content = "  \t  \n  \t\n";
        let repos = RepoConfig::parse_workspace_conf(config_content);
        assert_eq!(repos.len(), 0);
    }

    #[test]
    fn test_comments_and_empty_lines_mixed() {
        // Configuration with comments and empty lines
        let config_content = "\
# Comment


# Another comment


";
        let repos = RepoConfig::parse_workspace_conf(config_content);
        assert_eq!(repos.len(), 0);
    }

    #[test]
    fn test_config_missing_final_newline() {
        // Test that configuration with missing final newline still processes all repos
        // This tests the bug where `while read` in bash might skip the last line
        let config_content = "\
https://github.com/user/repo-a
https://github.com/user/repo-b
https://github.com/user/repo-c"; // No trailing newline

        let repos = RepoConfig::parse_workspace_conf(config_content);

        // All three repos should be parsed, including the last one without newline
        assert_eq!(repos.len(), 3);
        assert_eq!(repos[0].url, "https://github.com/user/repo-a");
        assert_eq!(repos[1].url, "https://github.com/user/repo-b");
        assert_eq!(repos[2].url, "https://github.com/user/repo-c");
    }

    #[test]
    fn test_to_config_value() {
        // Test formatting as git config value
        let repo1 = RepoConfig {
            url: "https://github.com/user/repo".to_string(),
            branch: "main".to_string(),
            git_ref: None,
        };
        assert_eq!(repo1.to_config_value(), "https://github.com/user/repo main");

        let repo2 = RepoConfig {
            url: "https://github.com/user/repo".to_string(),
            branch: "develop".to_string(),
            git_ref: Some("v1.0.0".to_string()),
        };
        assert_eq!(
            repo2.to_config_value(),
            "https://github.com/user/repo develop v1.0.0"
        );
    }

    #[test]
    fn test_parse_line_edge_cases() {
        // Test individual line parsing edge cases

        // URL only - should use default branch
        let repo = RepoConfig::parse_line("https://github.com/user/repo").unwrap();
        assert_eq!(repo.url, "https://github.com/user/repo");
        assert_eq!(repo.branch, "main");
        assert_eq!(repo.git_ref, None);

        // URL with branch
        let repo = RepoConfig::parse_line("https://github.com/user/repo develop").unwrap();
        assert_eq!(repo.branch, "develop");

        // URL with branch and ref
        let repo = RepoConfig::parse_line("https://github.com/user/repo main v1.0.0").unwrap();
        assert_eq!(repo.branch, "main");
        assert_eq!(repo.git_ref, Some("v1.0.0".to_string()));

        // Empty line should return None
        assert!(RepoConfig::parse_line("").is_none());

        // Comment should return None
        assert!(RepoConfig::parse_line("# comment").is_none());

        // Whitespace only should return None
        assert!(RepoConfig::parse_line("   \t   ").is_none());

        // Leading/trailing whitespace should be trimmed
        let repo = RepoConfig::parse_line("  https://github.com/user/repo  develop  ").unwrap();
        assert_eq!(repo.url, "https://github.com/user/repo");
        assert_eq!(repo.branch, "develop");
    }
}
