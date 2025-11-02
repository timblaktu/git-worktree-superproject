use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),

    #[error("IO operation failed: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid repository path: {0}")]
    InvalidPath(String),

    #[error("Worktree operation failed: {0}")]
    WorktreeError(String),

    #[error("Branch operation failed: {0}")]
    BranchError(String),

    #[error("Nix flake operation failed: {0}")]
    NixError(String),
}

pub type Result<T> = std::result::Result<T, WorkspaceError>;
