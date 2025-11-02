use crate::config::{Config, RepoConfig};
use crate::error::{Result, WorkspaceError};
use crate::git::GitOps;
use crate::workspace::{WorkspaceConfigBuilder, WorkspaceManager, WorkspaceManagerImpl};
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

        /// Old URL to replace
        #[arg(short, long)]
        old_url: String,

        /// New URL for the input
        #[arg(short = 'n', long)]
        new_url: String,
    },

    /// Manage per-workspace configurations
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Switch to a multi-repo workspace (creates if doesn't exist)
    Switch {
        /// Name of the workspace
        name: String,

        /// Configuration file with repository definitions
        #[arg(short = 'f', long, default_value = "workspace.conf")]
        config_file: PathBuf,
    },

    /// Synchronize (pull) all repositories in a workspace
    Sync {
        /// Name of the workspace
        name: String,
    },

    /// Execute a command in all repositories of a workspace
    Foreach {
        /// Name of the workspace
        name: String,

        /// Command to execute (use $name for repo name)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },

    /// Repair a broken repository in a workspace
    Repair {
        /// Name of the workspace
        workspace: String,

        /// Name of the repository to repair
        repo: String,
    },

    /// Regenerate workspace-specific flake.nix with input overrides
    RegenerateFlake {
        /// Name of the workspace (defaults to detecting from current directory)
        workspace: Option<String>,

        /// Source flake path (defaults to flake.nix in current directory)
        #[arg(short = 's', long, default_value = "flake.nix")]
        source: PathBuf,

        /// Output flake path (defaults to <workspace-dir>/flake.nix)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show configuration for a workspace
    Show {
        /// Workspace name (defaults to "main")
        #[arg(default_value = "main")]
        workspace: String,
    },

    /// Set repository configuration for a workspace
    Set {
        /// Workspace name
        workspace: String,

        /// Repository URL
        url: String,

        /// Branch name (optional)
        branch: Option<String>,

        /// Ref/tag (optional)
        git_ref: Option<String>,
    },

    /// Set default repository configuration
    SetDefault {
        /// Repository URL
        url: String,

        /// Branch name (optional)
        branch: Option<String>,

        /// Ref/tag (optional)
        git_ref: Option<String>,
    },

    /// Import configuration from workspace.conf file
    Import {
        /// Workspace name
        workspace: String,

        /// Source file path (defaults to workspace.conf)
        #[arg(default_value = "workspace.conf")]
        source_file: PathBuf,
    },

    /// Set flake input override for a workspace
    SetFlakeInput {
        /// Workspace name
        workspace: String,

        /// Input name (e.g., "nixpkgs")
        input_name: String,

        /// URL for the input
        url: String,

        /// Optional ref/branch (appended to URL)
        #[arg(short, long)]
        git_ref: Option<String>,
    },

    /// Set default flake input for all workspaces
    SetFlakeInputDefault {
        /// Input name (e.g., "nixpkgs")
        input_name: String,

        /// URL for the input
        url: String,

        /// Optional ref/branch (appended to URL)
        #[arg(short, long)]
        git_ref: Option<String>,
    },

    /// Show flake inputs for a workspace
    ShowFlakeInputs {
        /// Workspace name (defaults to "main")
        #[arg(default_value = "main")]
        workspace: String,
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
        Commands::Flake {
            file,
            input,
            old_url,
            new_url,
        } => {
            cmd_flake(file, input, old_url, new_url)?;
        }
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Show { workspace } => {
                cmd_config_show(config, workspace)?;
            }
            ConfigCommands::Set {
                workspace,
                url,
                branch,
                git_ref,
            } => {
                cmd_config_set(config, workspace, url, branch, git_ref)?;
            }
            ConfigCommands::SetDefault {
                url,
                branch,
                git_ref,
            } => {
                cmd_config_set_default(config, url, branch, git_ref)?;
            }
            ConfigCommands::Import {
                workspace,
                source_file,
            } => {
                cmd_config_import(config, workspace, source_file)?;
            }
            ConfigCommands::SetFlakeInput {
                workspace,
                input_name,
                url,
                git_ref,
            } => {
                cmd_config_set_flake_input(config, workspace, input_name, url, git_ref)?;
            }
            ConfigCommands::SetFlakeInputDefault {
                input_name,
                url,
                git_ref,
            } => {
                cmd_config_set_flake_input_default(config, input_name, url, git_ref)?;
            }
            ConfigCommands::ShowFlakeInputs { workspace } => {
                cmd_config_show_flake_inputs(config, workspace)?;
            }
        },
        Commands::Switch { name, config_file } => {
            cmd_switch(config, name, config_file)?;
        }
        Commands::Sync { name } => {
            cmd_sync(config, name)?;
        }
        Commands::Foreach { name, command } => {
            cmd_foreach(config, name, command)?;
        }
        Commands::Repair { workspace, repo } => {
            cmd_repair(config, workspace, repo)?;
        }
        Commands::RegenerateFlake {
            workspace,
            source,
            output,
        } => {
            cmd_regenerate_flake(config, workspace, source, output)?;
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

fn cmd_flake(file: PathBuf, input: String, old_url: String, new_url: String) -> Result<()> {
    info!(
        "Modifying Nix flake input '{}': {} -> {}",
        input, old_url, new_url
    );

    // Verify flake file exists
    if !file.exists() {
        return Err(WorkspaceError::ConfigError(format!(
            "Flake file not found: {}",
            file.display()
        )));
    }

    // Read the flake file
    let flake_content = std::fs::read_to_string(&file)
        .map_err(|e| WorkspaceError::ConfigError(format!("Failed to read flake file: {}", e)))?;

    // Use flake-input-modifier to perform the replacement
    let modified_content =
        flake_input_modifier::replace_flake_input_url(&flake_content, &input, &old_url, &new_url)
            .map_err(|e| WorkspaceError::NixError(format!("Failed to modify flake: {}", e)))?;

    // Write the modified content back to the file
    std::fs::write(&file, modified_content)
        .map_err(|e| WorkspaceError::ConfigError(format!("Failed to write flake file: {}", e)))?;

    println!("✓ Modified flake input '{}'", input);
    println!("  File: {}", file.display());
    println!("  Old URL: {}", old_url);
    println!("  New URL: {}", new_url);

    Ok(())
}

// ============================================================================
// Config Management Commands
// ============================================================================

fn cmd_config_show(config: Config, workspace: String) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;

    // Enable worktree config if not already enabled
    if !git.is_worktree_config_enabled() {
        git.enable_worktree_config()?;
    }

    let worktree_path = config.worktree_base.join(&workspace);

    // Check if workspace exists
    let workspace_exists = worktree_path.exists() && git.has_worktree(&workspace);

    println!("Configuration for workspace '{}':", workspace);
    println!();

    // Priority 1: Worktree-specific config
    if workspace_exists {
        let worktree_git = GitOps::open(&worktree_path)?;
        let worktree_configs = worktree_git.worktree_config_get_all("workspace.repo")?;

        if !worktree_configs.is_empty() {
            println!("Workspace-specific repositories:");
            for config_line in worktree_configs {
                println!("  {}", config_line);
            }
            println!();
            return Ok(());
        }
    }

    // Priority 2: Default (superproject) config
    let default_configs = git.config_get_all("workspace.repo")?;
    if !default_configs.is_empty() {
        println!("Default repositories (inherited):");
        for config_line in default_configs {
            println!("  {}", config_line);
        }
        println!();
        return Ok(());
    }

    // Priority 3: Legacy workspace.conf file
    let workspace_conf = std::env::current_dir()?.join("workspace.conf");
    if workspace_conf.exists() {
        println!("Legacy configuration (from workspace.conf):");
        let content = std::fs::read_to_string(&workspace_conf)?;
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                println!("  {}", trimmed);
            }
        }
        println!();
        return Ok(());
    }

    println!("No configuration found");
    Ok(())
}

fn cmd_config_set(
    config: Config,
    workspace: String,
    url: String,
    branch: Option<String>,
    git_ref: Option<String>,
) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;

    // Enable worktree config if not already enabled
    if !git.is_worktree_config_enabled() {
        git.enable_worktree_config()?;
    }

    let worktree_path = config.worktree_base.join(&workspace);

    // Check if workspace exists
    if !worktree_path.exists() {
        return Err(WorkspaceError::WorktreeError(format!(
            "Worktree '{}' does not exist. Create it first with 'workspace add {}'",
            workspace, workspace
        )));
    }

    if !git.has_worktree(&workspace) {
        return Err(WorkspaceError::WorktreeError(format!(
            "Worktree '{}' is not tracked by git",
            workspace
        )));
    }

    // Build config value: url branch [ref]
    let mut config_value = url.clone();
    let branch_name = branch.as_deref().unwrap_or("main");
    config_value.push(' ');
    config_value.push_str(branch_name);

    if let Some(ref_val) = &git_ref {
        config_value.push(' ');
        config_value.push_str(ref_val);
    }

    // Open worktree and set worktree-specific config
    let worktree_git = GitOps::open(&worktree_path)?;
    worktree_git.worktree_config_add("workspace.repo", &config_value)?;

    println!("Set repository config for workspace '{}':", workspace);
    println!("  {}", config_value);

    Ok(())
}

fn cmd_config_set_default(
    config: Config,
    url: String,
    branch: Option<String>,
    git_ref: Option<String>,
) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;

    // Enable worktree config if not already enabled
    if !git.is_worktree_config_enabled() {
        git.enable_worktree_config()?;
    }

    // Build config value: url branch [ref]
    let mut config_value = url.clone();
    let branch_name = branch.as_deref().unwrap_or("main");
    config_value.push(' ');
    config_value.push_str(branch_name);

    if let Some(ref_val) = &git_ref {
        config_value.push(' ');
        config_value.push_str(ref_val);
    }

    // Set in superproject config
    git.config_add("workspace.repo", &config_value)?;

    println!("Set default repository config:");
    println!("  {}", config_value);

    Ok(())
}

fn cmd_config_import(config: Config, workspace: String, source_file: PathBuf) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;

    // Enable worktree config if not already enabled
    if !git.is_worktree_config_enabled() {
        git.enable_worktree_config()?;
    }

    // Verify source file exists
    if !source_file.exists() {
        return Err(WorkspaceError::ConfigError(format!(
            "Source file not found: {}",
            source_file.display()
        )));
    }

    let worktree_path = config.worktree_base.join(&workspace);

    // Check if workspace exists, if not create it
    if !worktree_path.exists() || !git.has_worktree(&workspace) {
        info!("Workspace '{}' does not exist, creating it", workspace);

        // Create parent directory if needed
        if let Some(parent) = worktree_path.parent() {
            std::fs::create_dir_all(parent)?;
            info!("Created parent directory: {}", parent.display());
        }

        git.add_worktree(
            &workspace,
            &worktree_path,
            Some(&format!("workspace/{}", workspace)),
        )?;
    }

    // Open worktree
    let worktree_git = GitOps::open(&worktree_path)?;

    // Clear existing worktree-specific config
    let _ = worktree_git.worktree_config_unset_all("workspace.repo");

    // Read and import configuration
    let content = std::fs::read_to_string(&source_file)?;
    let mut imported_count = 0;

    println!("Importing configuration to workspace: {}", workspace);

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Import the line as-is using worktree-specific config
        worktree_git.worktree_config_add("workspace.repo", trimmed)?;
        println!("  Imported: {}", trimmed);
        imported_count += 1;
    }

    println!();
    println!("Import complete: {} repositories imported", imported_count);

    Ok(())
}

fn cmd_config_set_flake_input(
    config: Config,
    workspace: String,
    input_name: String,
    url: String,
    git_ref: Option<String>,
) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;

    // Enable worktree config if not already enabled
    if !git.is_worktree_config_enabled() {
        git.enable_worktree_config()?;
    }

    let worktree_path = config.worktree_base.join(&workspace);

    // Check if workspace exists, if not create it
    if !worktree_path.exists() || !git.has_worktree(&workspace) {
        info!("Workspace '{}' does not exist, creating it", workspace);

        // Create parent directory if needed
        if let Some(parent) = worktree_path.parent() {
            std::fs::create_dir_all(parent)?;
            info!("Created parent directory: {}", parent.display());
        }

        git.add_worktree(
            &workspace,
            &worktree_path,
            Some(&format!("workspace/{}", workspace)),
        )?;
    }

    // Build final URL with ref if provided
    let final_url = if let Some(ref_val) = &git_ref {
        // Handle different URL formats for appending refs
        if url.starts_with("github:") {
            format!("{}/{}", url, ref_val)
        } else if url.contains('?') {
            format!("{}&ref={}", url, ref_val)
        } else {
            format!("{}?ref={}", url, ref_val)
        }
    } else {
        url.clone()
    };

    // Open worktree and set flake input override
    let worktree_git = GitOps::open(&worktree_path)?;
    worktree_git.set_flake_input(&input_name, &final_url)?;

    println!("Set flake input for workspace '{}':", workspace);
    println!("  {}: {}", input_name, final_url);

    Ok(())
}

fn cmd_config_set_flake_input_default(
    config: Config,
    input_name: String,
    url: String,
    git_ref: Option<String>,
) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;

    // Enable worktree config if not already enabled
    if !git.is_worktree_config_enabled() {
        git.enable_worktree_config()?;
    }

    // Build final URL with ref if provided
    let final_url = if let Some(ref_val) = &git_ref {
        // Handle different URL formats for appending refs
        if url.starts_with("github:") {
            format!("{}/{}", url, ref_val)
        } else if url.contains('?') {
            format!("{}&ref={}", url, ref_val)
        } else {
            format!("{}?ref={}", url, ref_val)
        }
    } else {
        url.clone()
    };

    // Set default flake input in superproject config
    git.set_flake_input_default(&input_name, &final_url)?;

    println!("Set default flake input:");
    println!("  {}: {}", input_name, final_url);

    Ok(())
}

fn cmd_config_show_flake_inputs(config: Config, workspace: String) -> Result<()> {
    let git = GitOps::discover(&config.main_repo)?;
    let worktree_path = config.worktree_base.join(&workspace);

    println!("Flake inputs for workspace: {}", workspace);
    println!("=========================================");
    println!();

    // Check if workspace exists
    let workspace_exists = worktree_path.exists() && git.has_worktree(&workspace);

    // Priority 1: Workspace-specific overrides
    if workspace_exists {
        let worktree_git = GitOps::open(&worktree_path)?;
        let workspace_inputs = worktree_git.get_all_flake_inputs_worktree()?;

        if !workspace_inputs.is_empty() {
            println!("Workspace-specific input overrides:");
            for (name, url) in workspace_inputs {
                println!("  {}: {}", name, url);
            }
            println!();
        }
    }

    // Priority 2: Default overrides
    let default_inputs = git.get_all_flake_inputs_default()?;
    let has_default_inputs = !default_inputs.is_empty();
    if has_default_inputs {
        println!("Default input overrides (inherited):");
        for (name, url) in default_inputs {
            println!("  {}: {}", name, url);
        }
        println!();
    }

    // Note about original flake.nix inputs
    let flake_path = std::env::current_dir()?.join("flake.nix");
    if flake_path.exists() {
        println!("Original flake.nix inputs:");
        println!("  (These are used unless overridden above)");
        println!("  Run 'nix flake metadata' to see original inputs");
        println!();
    } else {
        if !workspace_exists {
            println!("Note: Workspace '{}' does not exist", workspace);
        }
        if !has_default_inputs {
            println!("No flake input overrides configured");
        }
    }

    Ok(())
}

// ============================================================================
// Multi-Repo Workspace Commands
// ============================================================================

fn cmd_switch(config: Config, name: String, config_file: PathBuf) -> Result<()> {
    info!(
        "Switching to workspace '{}' using config from {:?}",
        name, config_file
    );

    // Verify config file exists
    if !config_file.exists() {
        return Err(WorkspaceError::ConfigError(format!(
            "Configuration file not found: {}",
            config_file.display()
        )));
    }

    // Read and parse workspace.conf to get repository definitions
    let content = std::fs::read_to_string(&config_file)?;
    let repos = RepoConfig::parse_workspace_conf(&content);

    if repos.is_empty() {
        return Err(WorkspaceError::ConfigError(
            "No repositories found in configuration file".to_string(),
        ));
    }

    println!("Found {} repositories in configuration", repos.len());

    // Build workspace configuration manually
    let workspace_config = WorkspaceConfigBuilder::new()
        .worktree_base(config.worktree_base.clone())
        .build();

    // Replace the repos vector directly since there's no .repos() method
    let mut workspace_config = workspace_config;
    workspace_config.repos = repos;

    // Create workspace manager and switch to workspace
    let manager = WorkspaceManagerImpl::new_with_real_git();

    println!("Creating multi-repo workspace '{}'...", name);
    let report = manager
        .switch(&name, &workspace_config)
        .map_err(|e| WorkspaceError::WorktreeError(format!("Switch failed: {}", e)))?;

    // Display results
    println!("\nWorkspace '{}' created successfully!", name);

    if !report.repos_created.is_empty() {
        println!("\nRepositories created:");
        for repo_name in &report.repos_created {
            println!("  ✓ {}", repo_name);
        }
    }

    if !report.repos_skipped.is_empty() {
        println!("\nRepositories skipped (already exist):");
        for repo_name in &report.repos_skipped {
            println!("  → {}", repo_name);
        }
    }

    if !report.errors.is_empty() {
        println!("\nRepositories that failed:");
        for (repo_name, error) in &report.errors {
            println!("  ✗ {}: {}", repo_name, error);
        }
        return Err(WorkspaceError::WorktreeError(format!(
            "Failed to create {} repositories",
            report.errors.len()
        )));
    }

    println!(
        "\nWorkspace location: {}",
        config.worktree_base.join(&name).display()
    );
    Ok(())
}

fn cmd_sync(config: Config, name: String) -> Result<()> {
    info!("Synchronizing workspace '{}'", name);

    let workspace_path = config.worktree_base.join(&name);

    // Check if workspace exists
    if !workspace_path.exists() {
        return Err(WorkspaceError::WorktreeError(format!(
            "Workspace '{}' does not exist at {}",
            name,
            workspace_path.display()
        )));
    }

    // Create workspace manager and initialize worktree_base
    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.set_worktree_base(config.worktree_base.clone());

    println!("Synchronizing workspace '{}'...", name);
    let report = manager
        .sync(&name)
        .map_err(|e| WorkspaceError::WorktreeError(format!("Sync failed: {}", e)))?;

    // Display results
    if !report.repos_updated.is_empty() {
        println!("\nRepositories updated:");
        for repo_name in &report.repos_updated {
            println!("  ✓ {}", repo_name);
        }
    } else if !report.repos_pinned.is_empty() || !report.repos_failed.is_empty() {
        // Have pinned or failed repos, but no updates
        println!("\nNo repositories updated");
    } else {
        // No updates, no pinned, no failures = all up-to-date
        println!("\nAll repositories are up-to-date");
    }

    if !report.repos_pinned.is_empty() {
        println!("\nPinned repositories (skipped):");
        for repo_name in &report.repos_pinned {
            println!("  → {}", repo_name);
        }
    }

    if !report.repos_failed.is_empty() {
        println!("\nRepositories that failed:");
        for (repo_name, error) in &report.repos_failed {
            println!("  ✗ {}: {}", repo_name, error);
        }
        return Err(WorkspaceError::WorktreeError(format!(
            "Failed to sync {} repositories",
            report.repos_failed.len()
        )));
    }

    Ok(())
}

fn cmd_foreach(config: Config, name: String, command: Vec<String>) -> Result<()> {
    info!("Executing command across workspace '{}'", name);

    if command.is_empty() {
        return Err(WorkspaceError::ConfigError(
            "No command specified".to_string(),
        ));
    }

    let workspace_path = config.worktree_base.join(&name);

    // Check if workspace exists
    if !workspace_path.exists() {
        return Err(WorkspaceError::WorktreeError(format!(
            "Workspace '{}' does not exist at {}",
            name,
            workspace_path.display()
        )));
    }

    // Create workspace manager and initialize worktree_base
    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.set_worktree_base(config.worktree_base.clone());

    let command_str = command.join(" ");
    println!(
        "Executing '{}' across workspace '{}'...\n",
        command_str, name
    );
    let result = manager
        .foreach(&name, &command)
        .map_err(|e| WorkspaceError::WorktreeError(format!("Foreach failed: {}", e)))?;

    // Display results
    for output in &result.outputs {
        println!("Repository: {}", output.repo_name);
        let success = output.exit_code == 0;
        if success {
            println!("  Status: ✓ Success");
            if !output.stdout.is_empty() {
                println!("  Output:");
                for line in output.stdout.lines() {
                    println!("    {}", line);
                }
            }
        } else {
            println!("  Status: ✗ Failed (exit code: {})", output.exit_code);
            if !output.stderr.is_empty() {
                println!("  Error:");
                for line in output.stderr.lines() {
                    println!("    {}", line);
                }
            }
        }
        println!();
    }

    // Check if any commands failed
    let failed_count = result.outputs.iter().filter(|o| o.exit_code != 0).count();
    if failed_count > 0 {
        return Err(WorkspaceError::WorktreeError(format!(
            "Command failed in {} repositories",
            failed_count
        )));
    }

    println!("Command executed successfully in all repositories");
    Ok(())
}

fn cmd_repair(config: Config, workspace: String, repo: String) -> Result<()> {
    info!(
        "Repairing repository '{}' in workspace '{}'",
        repo, workspace
    );

    let workspace_path = config.worktree_base.join(&workspace);

    // Check if workspace exists
    if !workspace_path.exists() {
        return Err(WorkspaceError::WorktreeError(format!(
            "Workspace '{}' does not exist at {}",
            workspace,
            workspace_path.display()
        )));
    }

    // Create workspace manager and initialize worktree_base
    let manager = WorkspaceManagerImpl::new_with_real_git();
    manager.set_worktree_base(config.worktree_base.clone());

    println!(
        "Attempting to repair repository '{}' in workspace '{}'...\n",
        repo, workspace
    );
    let report = manager
        .repair(&workspace, &repo)
        .map_err(|e| WorkspaceError::WorktreeError(format!("Repair failed: {}", e)))?;

    // Display results with emoji indicators
    if report.success {
        println!("✓ Repair successful: {}", report.repo_name);
        println!("  Action: {:?}", report.action_taken);
        println!("  Details: {}", report.message);
    } else {
        println!("✗ Repair failed: {}", report.repo_name);
        println!("  Attempted action: {:?}", report.action_taken);
        println!("  Error: {}", report.message);
        return Err(WorkspaceError::WorktreeError(format!(
            "Failed to repair repository '{}': {}",
            repo, report.message
        )));
    }

    Ok(())
}

fn cmd_regenerate_flake(
    config: Config,
    workspace: Option<String>,
    source: PathBuf,
    output: Option<PathBuf>,
) -> Result<()> {
    // Determine workspace name
    let workspace_name = if let Some(name) = workspace {
        name
    } else {
        // Try to detect workspace from current directory
        let current_dir = std::env::current_dir()?;
        let worktree_base = &config.worktree_base;

        if current_dir.starts_with(worktree_base) {
            // Extract workspace name from path
            let relative_path = current_dir.strip_prefix(worktree_base).map_err(|_| {
                WorkspaceError::ConfigError(
                    "Could not determine workspace from current directory".to_string(),
                )
            })?;

            // Get the first component (workspace name)
            relative_path
                .components()
                .next()
                .and_then(|c| c.as_os_str().to_str())
                .ok_or_else(|| {
                    WorkspaceError::ConfigError(
                        "Could not determine workspace from current directory".to_string(),
                    )
                })?
                .to_string()
        } else {
            return Err(WorkspaceError::ConfigError(
                "Not in a workspace directory. Please specify workspace name or cd to a workspace."
                    .to_string(),
            ));
        }
    };

    info!("Regenerating flake.nix for workspace '{}'", workspace_name);

    let worktree_path = config.worktree_base.join(&workspace_name);

    // Check if workspace exists
    if !worktree_path.exists() {
        return Err(WorkspaceError::WorktreeError(format!(
            "Workspace '{}' does not exist at {}",
            workspace_name,
            worktree_path.display()
        )));
    }

    // Verify source flake exists
    if !source.exists() {
        return Err(WorkspaceError::ConfigError(format!(
            "Source flake not found: {}",
            source.display()
        )));
    }

    // Determine output path
    let output_path = output.unwrap_or_else(|| worktree_path.join("flake.nix"));

    println!("Regenerating flake.nix for workspace: {}", workspace_name);
    println!("  Source: {}", source.display());
    println!("  Output: {}", output_path.display());
    println!();

    // Read source flake
    let source_content = std::fs::read_to_string(&source)
        .map_err(|e| WorkspaceError::ConfigError(format!("Failed to read source flake: {}", e)))?;

    // Open workspace git repository
    let workspace_git = GitOps::open(&worktree_path)?;

    // Generate workspace-specific flake with overrides
    let modified_content = workspace_git.generate_workspace_flake(&source_content)?;

    // Write output flake
    std::fs::write(&output_path, modified_content)
        .map_err(|e| WorkspaceError::ConfigError(format!("Failed to write output flake: {}", e)))?;

    println!("✓ Workspace flake.nix regenerated successfully");
    println!("  Location: {}", output_path.display());

    // Show which overrides were applied
    let workspace_inputs = workspace_git.get_all_flake_inputs_worktree()?;
    let default_inputs = workspace_git.get_all_flake_inputs_default()?;

    if !workspace_inputs.is_empty() || !default_inputs.is_empty() {
        println!("\nApplied input overrides:");
        for (name, url) in workspace_inputs {
            println!("  {} = {} (workspace-specific)", name, url);
        }
        for (name, url) in default_inputs {
            println!("  {} = {} (default)", name, url);
        }
    } else {
        println!("\nNo input overrides configured (flake copied as-is)");
    }

    Ok(())
}
