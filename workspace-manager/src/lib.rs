//! Workspace Manager - Multi-repository workspace management
//!
//! This library provides functionality for managing multi-repository workspaces
//! using git worktrees.

pub mod cli;
pub mod config;
pub mod error;
pub mod fs;
pub mod git;
pub mod workspace;
