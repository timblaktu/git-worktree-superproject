use crate::error::{Result, WorkspaceError};
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

        toml::from_str(&contents)
            .map_err(|e| WorkspaceError::ConfigError(format!("Failed to parse config: {}", e)))
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
            worktree_base: PathBuf::from("~/.worktrees"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
}
