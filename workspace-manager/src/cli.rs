use crate::config::Config;
use crate::error::{Result, WorkspaceError};
use crate::git::GitOps;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{debug, info};

/// Unified workspace manager for git worktrees with Nix flake support
#[derive(Parser, Debug)]
#[command(name = "workspace")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize workspace configuration
    Init {
        /// Base directory for worktrees
        #[arg(short, long)]
        worktree_base: Option<PathBuf>,

        /// Main repository path
        #[arg(short, long)]
        repo: Option<PathBuf>,
    },

    /// List all worktrees
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Create a new worktree
    Add {
        /// Name of the worktree
        name: String,

        /// Branch name (creates new branch if doesn't exist)
        #[arg(short, long)]
        branch: Option<String>,

        /// Path for the worktree (defaults to <worktree_base>/<name>)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Remove a worktree
    Remove {
        /// Name of the worktree to remove
        name: String,

        /// Force removal even if worktree has uncommitted changes
        #[arg(short, long)]
        force: bool,
    },

    /// Show worktree information
    Info {
        /// Name of the worktree
        name: String,
    },

    /// List all branches
    Branches {
        /// Include remote branches
        #[arg(short, long)]
        remote: bool,
    },

    /// Show repository status
    Status,

    /// Modify Nix flake inputs (integration with flake-input-modifier)
    Flake {
        /// Flake file path
        #[arg(short, long, default_value = "flake.nix")]
        file: PathBuf,

        /// Input name to modify
        #[arg(short, long)]
        input: String,

        /// New URL for the input
        #[arg(short, long)]
        url: String,
    },
}

/// Parse command-line arguments
pub fn parse_args() -> Args {
    Args::parse()
}

/// Execute the appropriate command based on parsed arguments
pub fn execute_command(args: Args) -> Result<()> {
    // Load configuration
    let config = if let Some(config_path) = &args.config {
        Config::load(config_path)?
    } else {
        Config::detect()?
    };

    debug!("Loaded configuration: {:?}", config);

    match args.command {
        Commands::Init {
            worktree_base,
            repo,
        } => {
            cmd_init(config, worktree_base, repo)?;
        }
        Commands::List { detailed } => {
            cmd_list(config, detailed)?;
        }
        Commands::Add { name, branch, path } => {
            cmd_add(config, name, branch, path)?;
        }
        Commands::Remove { name, force } => {
            cmd_remove(config, name, force)?;
        }
        Commands::Info { name } => {
            cmd_info(config, name)?;
        }
        Commands::Branches { remote } => {
            cmd_branches(config, remote)?;
        }
        Commands::Status => {
            cmd_status(config)?;
        }
        Commands::Flake { file, input, url } => {
            cmd_flake(file, input, url)?;
        }
    }

    Ok(())
}

fn cmd_init(
    mut config: Config,
    worktree_base: Option<PathBuf>,
    repo: Option<PathBuf>,
) -> Result<()> {
    info!("Initializing workspace configuration");

    if let Some(base) = worktree_base {
        config.worktree_base = base;
    }

    if let Some(repo_path) = repo {
        config.main_repo = repo_path;
    }

    // Save configuration
    let config_path = dirs::home_dir()
        .ok_or_else(|| WorkspaceError::ConfigError("Could not find home directory".to_string()))?
        .join(".config/workspace/config.toml");

    // Create parent directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    config.save(&config_path)?;

    println!(
        "Initialized workspace configuration at: {}",
        config_path.display()
    );
    println!("Worktree base: {}", config.worktree_base.display());
    println!("Main repository: {}", config.main_repo.display());

    Ok(())
}

fn cmd_list(config: Config, detailed: bool) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;
    let worktrees = git.list_worktrees()?;

    if worktrees.is_empty() {
        println!("No worktrees found");
        return Ok(());
    }

    println!("Worktrees:");
    for name in worktrees {
        if detailed {
            if let Ok(info) = git.worktree_info(&name) {
                // Get branch and status information for the worktree
                let branch = if info.path.exists() {
                    match GitOps::open(&info.path) {
                        Ok(wt_git) => match wt_git.current_branch() {
                            Ok(b) => format!("on branch {}", b),
                            Err(_) => "detached HEAD".to_string(),
                        },
                        Err(_) => "unknown branch".to_string(),
                    }
                } else {
                    "directory missing".to_string()
                };

                let status = if info.path.exists() {
                    match git.worktree_status(&info.path) {
                        Ok(s) => {
                            if s.is_clean() {
                                "clean".to_string()
                            } else {
                                format!(
                                    "M:{} A:{} D:{} U:{}",
                                    s.modified, s.added, s.deleted, s.untracked
                                )
                            }
                        }
                        Err(_) => "unknown".to_string(),
                    }
                } else {
                    "n/a".to_string()
                };

                println!("  {} ({})", name, info.path.display());
                println!("    Branch: {}", branch);
                println!("    Status: {}", status);
                println!("    Locked: {}", info.is_locked);
                println!("    Valid: {}", info.is_valid);
            }
        } else {
            // Simple list - show name and basic info
            if let Ok(info) = git.worktree_info(&name) {
                let branch_marker = if info.path.exists() {
                    match GitOps::open(&info.path) {
                        Ok(wt_git) => match wt_git.current_branch() {
                            Ok(b) => format!(" [{}]", b),
                            Err(_) => String::new(),
                        },
                        Err(_) => String::new(),
                    }
                } else {
                    " [missing]".to_string()
                };
                println!("  {}{}", name, branch_marker);
            } else {
                println!("  {}", name);
            }
        }
    }

    Ok(())
}

fn cmd_add(
    config: Config,
    name: String,
    branch: Option<String>,
    path: Option<PathBuf>,
) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;

    // Check if worktree with this name already exists
    if git.has_worktree(&name) {
        return Err(WorkspaceError::WorktreeError(format!(
            "Worktree '{}' already exists",
            name
        )));
    }

    // Determine worktree path
    let worktree_path = if let Some(p) = path {
        p
    } else {
        config.worktree_base.join(&name)
    };

    // Check if path already exists
    if worktree_path.exists() {
        return Err(WorkspaceError::WorktreeError(format!(
            "Path already exists: {}",
            worktree_path.display()
        )));
    }

    // Create parent directory if needed
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
        info!("Created parent directory: {}", parent.display());
    }

    // Determine branch to use
    let branch_to_use = branch.as_deref();

    // If branch is specified, check if it already exists
    if let Some(branch_name) = branch_to_use {
        let branches = git.list_branches(None)?;
        if branches.contains(&branch_name.to_string()) {
            info!("Branch '{}' already exists, will check it out", branch_name);
        } else {
            info!("Branch '{}' will be created", branch_name);
        }
    }

    // Add the worktree
    debug!(
        "Creating worktree '{}' at {}",
        name,
        worktree_path.display()
    );
    git.add_worktree(&name, &worktree_path, branch_to_use)?;

    // Verify the worktree was created successfully
    let info = git.worktree_info(&name)?;
    if !info.is_valid {
        return Err(WorkspaceError::WorktreeError(format!(
            "Worktree '{}' was created but is not valid",
            name
        )));
    }

    // Verify directory exists
    if !worktree_path.exists() {
        return Err(WorkspaceError::WorktreeError(format!(
            "Worktree directory was not created: {}",
            worktree_path.display()
        )));
    }

    println!(
        "✓ Added worktree '{}' at: {}",
        name,
        worktree_path.display()
    );
    if let Some(branch_name) = branch {
        println!("  Branch: {}", branch_name);
    }
    println!("  Status: Valid");

    Ok(())
}

fn cmd_remove(config: Config, name: String, force: bool) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;

    // Check if worktree exists
    if !git.has_worktree(&name) {
        return Err(WorkspaceError::WorktreeError(format!(
            "Worktree '{}' does not exist",
            name
        )));
    }

    // Get worktree information
    let info = git.worktree_info(&name)?;
    let worktree_path = info.path.clone();

    // Check if worktree is locked
    if info.is_locked {
        return Err(WorkspaceError::WorktreeError(format!(
            "Worktree '{}' is locked. Unlock it before removing",
            name
        )));
    }

    // Check for uncommitted changes unless --force is specified
    if !force && worktree_path.exists() {
        match git.worktree_status(&worktree_path) {
            Ok(status) => {
                if !status.is_clean() {
                    return Err(WorkspaceError::WorktreeError(format!(
                        "Worktree '{}' has uncommitted changes:\n\
                         Modified: {}, Added: {}, Deleted: {}, Untracked: {}\n\
                         Use --force to remove anyway",
                        name, status.modified, status.added, status.deleted, status.untracked
                    )));
                }
            }
            Err(e) => {
                debug!("Could not check worktree status: {}", e);
                // If we can't check status, warn but don't block unless not forcing
                if !force {
                    return Err(WorkspaceError::WorktreeError(format!(
                        "Could not verify worktree status: {}. Use --force to remove anyway",
                        e
                    )));
                }
            }
        }
    }

    // Remove the worktree from git
    info!("Removing worktree '{}' from git", name);
    git.remove_worktree(&name)?;

    // Remove the directory from filesystem if it exists
    if worktree_path.exists() {
        info!("Removing directory: {}", worktree_path.display());
        std::fs::remove_dir_all(&worktree_path)?;
        println!(
            "✓ Removed worktree '{}' and directory: {}",
            name,
            worktree_path.display()
        );
    } else {
        println!("✓ Removed worktree '{}' (directory already removed)", name);
    }

    Ok(())
}

fn cmd_info(config: Config, name: String) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;
    let info = git.worktree_info(&name)?;

    println!("Worktree: {}", info.name);
    println!("Path: {}", info.path.display());
    println!("Locked: {}", info.is_locked);
    println!("Valid: {}", info.is_valid);

    Ok(())
}

fn cmd_branches(config: Config, _remote: bool) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;
    let branches = git.list_branches(None)?;

    if branches.is_empty() {
        println!("No branches found");
        return Ok(());
    }

    let current = git.current_branch().ok();

    println!("Branches:");
    for branch in branches {
        if Some(&branch) == current.as_ref() {
            println!("* {}", branch);
        } else {
            println!("  {}", branch);
        }
    }

    Ok(())
}

fn cmd_status(config: Config) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;
    let status = git.status_summary()?;

    println!("Repository status:");
    println!("  Modified: {}", status.modified);
    println!("  Added: {}", status.added);
    println!("  Deleted: {}", status.deleted);
    println!("  Untracked: {}", status.untracked);

    if status.is_clean() {
        println!("\nRepository is clean");
    }

    Ok(())
}

fn cmd_flake(file: PathBuf, input: String, url: String) -> Result<()> {
    info!("Modifying Nix flake input: {} -> {}", input, url);

    // Use the flake-input-modifier functionality
    // This is a placeholder - will integrate with the actual flake-input-modifier library
    println!("Flake modification:");
    println!("  File: {}", file.display());
    println!("  Input: {}", input);
    println!("  New URL: {}", url);

    println!("\nNote: Flake modification integration coming in Phase 2");

    Ok(())
}
