use crate::error::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// File system operations for workspace management
pub struct FileSystem;

impl FileSystem {
    /// Create a directory with all parent directories
    #[allow(dead_code)] // Used in tests
    pub fn create_dir_all(path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    /// Remove a directory and all its contents
    #[allow(dead_code)] // Used in tests
    pub fn remove_dir_all(path: &Path) -> Result<()> {
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    /// Check if a path exists
    #[allow(dead_code)] // Used in tests
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    /// Check if a path is a directory
    #[allow(dead_code)] // Used in tests
    pub fn is_dir(path: &Path) -> bool {
        path.is_dir()
    }

    /// Expand tilde in path
    pub fn expand_tilde(path: &Path) -> PathBuf {
        if let Ok(stripped) = path.strip_prefix("~") {
            if let Some(home) = dirs::home_dir() {
                return home.join(stripped);
            }
        }
        path.to_path_buf()
    }

    /// Find files matching a pattern in a directory
    #[allow(dead_code)] // Future feature
    pub fn find_files(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(dir).follow_links(true) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if entry.file_type().is_file() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.contains(pattern) {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }

    /// Get the canonical absolute path
    #[allow(dead_code)] // Future feature
    pub fn canonicalize(path: &Path) -> Result<PathBuf> {
        let expanded = Self::expand_tilde(path);
        Ok(std::fs::canonicalize(expanded)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_remove_dir() {
        let temp = tempdir().unwrap();
        let test_path = temp.path().join("test/nested/dir");

        FileSystem::create_dir_all(&test_path).unwrap();
        assert!(FileSystem::exists(&test_path));
        assert!(FileSystem::is_dir(&test_path));

        FileSystem::remove_dir_all(&test_path.parent().unwrap().parent().unwrap()).unwrap();
        assert!(!FileSystem::exists(&test_path));
    }

    #[test]
    fn test_expand_tilde() {
        let path = Path::new("~/test");
        let expanded = FileSystem::expand_tilde(path);
        assert!(!expanded.starts_with("~"));
    }
}
