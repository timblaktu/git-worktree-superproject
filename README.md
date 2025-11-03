# workspace: Multi-Repository Management with Git Worktrees

> **A high-performance Rust tool for managing multiple git repositories as a cohesive workspace**

## Overview

`workspace` is a command-line tool that manages independent "workspaces" using git worktrees, where each workspace contains linked working trees of all your repositories at consistent branches or specific versions.

### Why Workspace?

Git submodules were designed for managing external dependencies with independent versioning. When repositories need to move together as a unit (synchronized branches, coordinated releases), submodules create unnecessary complexity. `workspace` provides a simpler alternative.

### Key Features

- ✅ **Git worktree efficiency** - Shared git objects reduce disk usage
- ✅ **Type-safe** - Rust implementation with compile-time guarantees
- ✅ **Flexible configuration** - Per-workspace and default configurations
- ✅ **Version-controlled configs** - Configuration tracked in git
- ✅ **Flexible pinning** - Mix branch-tracking and version-pinned repositories
- ✅ **Isolated workspaces** - Each workspace uses independent worktrees
- ✅ **Parallel development** - Multiple worktrees for different features
- ✅ **Nix flake integration** - AST-based flake input modification with 3-tier inheritance
- ✅ **Comprehensive testing** - 241 tests ensuring reliability

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/workspace.git
cd workspace

# Build release binary
cargo build --release

# Install to your PATH
cp target/release/workspace ~/.local/bin/
# or
sudo cp target/release/workspace /usr/local/bin/
```

### Requirements

- Rust toolchain (1.70+)
- Git 2.20+

## Quick Start

1. **Initialize your project:**
```bash
mkdir my-project && cd my-project
git init
```

2. **Configure your repositories:**
```bash
# Add repositories to default configuration
workspace config set-default https://github.com/myorg/app.git
workspace config set-default https://github.com/myorg/backend.git
workspace config set-default https://github.com/myorg/shared-lib.git

# Repository with specific branch preference
workspace config set-default https://github.com/myorg/firmware.git stable

# Pinned dependencies (specific versions)
workspace config set-default https://github.com/vendor/sdk.git main v1.2.0
```

3. **Create your first workspace:**
```bash
workspace switch main
cd worktrees/main
# All repositories are now available and ready for development
```

4. **Add to `.gitignore`:**
```bash
echo "worktrees/" >> .gitignore
echo "repos/" >> .gitignore
```

## Core Commands

### Workspace Management

- `workspace switch <name>` - Switch to or create a workspace
- `workspace sync [name]` - Synchronize workspace repositories
- `workspace list` - List all workspaces
- `workspace clean <name>` - Remove a workspace
- `workspace status` - Show status of all workspaces
- `workspace foreach <command>` - Run command in all workspace repositories

### Configuration

- `workspace config set <workspace> <url> [branch] [ref]` - Set workspace-specific config
- `workspace config set-default <url> [branch] [ref]` - Set default repository config
- `workspace config show [workspace]` - Display effective configuration
- `workspace config import <workspace> <file>` - Import from configuration file

### Git Worktree Operations

- `workspace init` - Initialize repository for worktrees
- `workspace add <name> [branch]` - Add a worktree
- `workspace remove <name>` - Remove a worktree
- `workspace info <name>` - Show worktree information
- `workspace branches` - List all branches
- `workspace worktree-status` - Show git status summary

### Maintenance

- `workspace repair <workspace> <repo>` - Repair broken repositories

### Nix Flake Integration

When a `flake.nix` is detected in your project:

- `workspace config set-flake-input <workspace> <input> <url>` - Override flake input for workspace
- `workspace config set-flake-input-default <input> <url>` - Set default flake input
- `workspace config show-flake-inputs [workspace]` - Show flake configuration
- `workspace regenerate-flake [workspace]` - Generate workspace-specific flake

## Directory Structure

```
my-project/
├── .git/                  # Superproject repository
│   ├── config             # Default configurations (workspace.repo.*)
│   └── worktrees/         # Git worktree metadata
├── repos/                 # Central bare repositories (cache)
└── worktrees/             # All workspaces
    ├── main/              # Main workspace (git worktree)
    │   ├── .git           # Worktree git file
    │   ├── app/           # Repository checkout
    │   ├── backend/       # Repository checkout
    │   └── shared-lib/    # Repository checkout
    └── feature-x/         # Feature workspace (git worktree)
        ├── .git
        ├── app/
        ├── backend/
        └── shared-lib/
```

## Configuration System

### 3-Tier Configuration Inheritance

1. **Workspace-specific** - Highest priority, stored in `.git/worktrees/<name>/config.worktree`
2. **Default configuration** - Shared across workspaces, stored in `.git/config`
3. **Fallback** - Uses workspace branch name if no configuration exists

### Example: Mixed Repository Configuration

```bash
# Set defaults for all workspaces
workspace config set-default https://github.com/org/app.git
workspace config set-default https://github.com/org/backend.git

# Override for specific workspace
workspace config set feature-x backend feature-branch

# Pin a dependency to specific version
workspace config set-default https://github.com/vendor/lib.git main v2.1.0
```

## Nix Flake Integration

The tool provides AST-based modification of Nix flake inputs, preserving formatting and structure.

### Use Case: Multi-Context Development

```bash
# Configure development context with local forks
workspace config set-flake-input dev nixpkgs "git+file:///home/user/nixpkgs?ref=my-feature"
workspace config set-flake-input dev home-manager "git+file:///home/user/home-manager?ref=my-module"

# Configure upstream context for testing
workspace config set-flake-input upstream nixpkgs "github:NixOS/nixpkgs/nixos-unstable"
workspace config set-flake-input upstream home-manager "github:nix-community/home-manager"

# Switch contexts instantly
workspace switch dev         # Uses local forks
workspace switch upstream    # Uses upstream packages

# Show effective flake configuration
workspace config show-flake-inputs dev
```

## Testing

Run the comprehensive test suite:

```bash
# All tests
cargo test --workspace

# Specific test categories
cargo test --workspace unit
cargo test --workspace integration
cargo test --workspace cli
```

**Test Coverage**: 241 tests (100% passing)
- Unit tests: 81
- Integration tests: 78
- CLI tests: 51
- Property tests: 4
- Others: 27

## Development

### Build

```bash
cargo build              # Debug build
cargo build --release    # Optimized build
```

### Run

```bash
cargo run -- <command>   # Run with cargo
./target/debug/workspace <command>    # Run debug binary
./target/release/workspace <command>  # Run release binary
```

### Lint

```bash
cargo clippy --workspace
cargo fmt --check
```

## Contributing

1. Fork the repository
2. Create your feature branch: `git checkout -b feature-amazing`
3. Add tests for new functionality
4. Ensure tests pass: `cargo test --workspace`
5. Ensure code quality: `cargo clippy --workspace`
6. Commit your changes: `git commit -m 'Add amazing feature'`
7. Push to branch: `git push origin feature-amazing`
8. Open a Pull Request

## Architecture

The tool uses a trait-based architecture for testability:

- **GitOps** - Git operations abstraction (libgit2-rs)
- **WorkspaceManager** - Multi-repository workspace management
- **RepositoryOps** - Repository operations (mockable for tests)
- **Config** - Configuration parsing and management
- **CLI** - Command-line interface (clap)

## License

MIT License - see LICENSE file for details.

## Project Status

✅ **Production Ready** - All core features implemented and tested

- 15 commands fully functional
- 241 tests passing (100%)
- Zero compiler warnings
- Comprehensive error handling
- Full Nix flake workflow support
