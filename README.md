# wt-super: Multi-Repository Management with Git Worktrees

> **A lightweight alternative to Git submodules for managing multiple repositories as a cohesive unit**

## Overview

`wt-super` is a simple bash script that replaces Git submodules with a more flexible and intuitive approach to multi-repository development. Instead of dealing with submodule complexity, it manages independent "workspaces" using git worktrees where each contains linked working trees of all your repositories at consistent branches or specific versions.

### Why Not Submodules?

Git submodules were designed for managing external dependencies with independent versioning. When all repositories need to move together as a unit (synchronized branches, coordinated releases), submodules create unnecessary complexity:

- ❌ **Detached HEAD states** that confuse developers
- ❌ **Complex commands** like `git submodule update --init --recursive`
- ❌ **Nested .git directories** and .gitmodules management overhead  
- ❌ **Commit pollution** from submodule pointer updates
- ❌ **Fragile workflows** that break easily

### The wt-super Solution

- ✅ **Git worktree efficiency** - Shared git objects reduce disk usage
- ✅ **Flexible configuration** - Per-workspace or shared configurations
- ✅ **Version-controlled configs** - Workspace settings tracked in git
- ✅ **Flexible pinning** - Mix branch-tracking and version-pinned repositories
- ✅ **Isolated workspaces** - Each workspace uses independent worktrees
- ✅ **Clean Git history** - No submodule pointer commits
- ✅ **Intuitive commands** - Easy to learn and debug
- ✅ **Parallel development** - Multiple worktrees for different features

## Quick Start

1. **Create your project:**
```bash
mkdir my-project && cd my-project
```

2. **Download the workspace script:**
```bash
curl -o workspace https://raw.githubusercontent.com/example/wt-super/main/workspace
chmod +x workspace
```

3. **Configure your repositories using templates:**
```bash
# Set up repository templates (replaces workspace.conf)
./workspace config set-default git@github.com:myorg/app.git
./workspace config set-default git@github.com:myorg/backend.git  
./workspace config set-default git@github.com:myorg/shared-lib.git

# Repository with specific branch preference
./workspace config set-default git@github.com:myorg/firmware.git stable

# Pinned dependencies (read-only, specific versions)
./workspace config set-default https://github.com/vendor/sdk.git main v1.2.0
./workspace config set-default https://github.com/third-party/lib.git master 3a4b5c6
```

4. **Create your first workspace:**
```bash
./workspace switch main
cd worktrees/main
# All repositories are now available and ready for development
```

5. **Add to `.gitignore`:**
```bash
echo "worktrees/" >> .gitignore
```

## Directory Structure

```
my-project/
├── workspace               # Management script  
├── .gitignore             # Ignore worktrees and repos directories
├── .git/                  # Superproject git repository
│   ├── config             # Template configurations (workspace.repo)
│   │                      # Workspace overrides (workspace.override.*)
│   └── worktrees/         # Git worktree metadata
├── repos/                 # Central bare repositories (internal cache)
└── worktrees/             # All workspaces live here
    ├── main/              # Main branch workspace (git worktree)
    │   ├── .git           # Worktree git file
    │   ├── app/           # First repository
    │   ├── backend/       # Second repository  
    │   └── shared-lib/    # Third repository
    └── feature-auth/      # Feature branch workspace (git worktree)
        ├── .git           # Worktree git file
        ├── app/
        ├── backend/
        └── shared-lib/
```

## Configuration

### Per-Workspace Configuration (NEW)

**wt-super now supports per-workspace repository configurations!** Each workspace can have completely different repository specifications, enabling advanced workflows:

- Different branches for different workspaces
- Workspace-specific repository subsets
- Independent version pinning per workspace
- Configuration tracked in git history

#### Configuration Priority

1. **Workspace-specific overrides** (superproject git config) - Highest priority
2. **Template defaults** (superproject git config) - Inherited by all workspaces

> **Note**: workspace.conf files are no longer supported. Use `workspace config` commands for all configuration.

#### Setting Configurations

```bash
# Set template repositories for all workspaces
./workspace config set-default https://github.com/org/app.git
./workspace config set-default https://github.com/org/backend.git main

# Set workspace-specific overrides
./workspace config set feature-x app feature-branch
./workspace config set feature-x backend develop

# Show effective configuration for a workspace
./workspace config show feature-x
```

### ⚠️  Deprecated: workspace.conf Format 

> **DEPRECATED**: The `workspace.conf` file format has been removed and is no longer supported. Use `workspace config` commands instead.

For reference, the old format used a simple space-separated syntax:

```bash
# workspace.conf
# Format: url [branch] [ref]
# - url: Git repository URL (required)
# - branch: Branch to track (optional, defaults to workspace branch)  
# - ref: Specific commit/tag (optional, makes repo read-only)

# Standard repositories - track workspace branches
git@github.com:org/app.git
git@github.com:org/lib-core.git
git@github.com:org/lib-utils.git

# Repository with specific branch preference
git@github.com:org/firmware.git stable

# Pinned dependencies (read-only)
https://github.com/espressif/esp-idf.git release/v5.1 v5.1.2
https://github.com/third-party/lib.git main 3a4b5c6
```

### Configuration Rules

- **Lines starting with `#`** are comments
- **Empty lines** are ignored
- **URL only**: Repository tracks the workspace branch name
- **URL + branch**: Repository always uses the specified branch  
- **URL + branch + ref**: Repository is pinned to specific commit/tag (read-only)

## Commands

### `workspace switch [branch]`
Switch to a workspace for the specified branch, creating it if needed (defaults to `main`).

```bash
./workspace switch                    # Switch to main workspace
./workspace switch feature-payment    # Switch to feature branch workspace
./workspace switch release-v2.0       # Switch to release workspace
```

**Behavior:**
- Creates `worktrees/[branch]` directory if it doesn't exist
- Creates git worktrees for all repositories (shared git objects in `repos/` directory)
- Repositories track workspace branch unless configured otherwise
- Pinned repositories checkout to specific refs (detached HEAD)
- Skips repositories that already exist

### `workspace sync [branch]`
Synchronize repositories in the specified workspace (defaults to current directory).

```bash
cd worktrees/feature-payment
../../workspace sync              # Sync current workspace
./workspace sync feature-payment  # Sync specific workspace
```

**Behavior:**
- Fetches latest changes from origin
- Updates branch-tracking repositories with `git pull --ff-only`
- Skips pinned repositories (they remain at fixed versions)
- Reports warnings for missing repositories

### `workspace status`
Show status of all workspaces and their repositories.

```bash
./workspace status
```

**Output:**
```
Workspace Status
================

main
----------------------------------------
  app:                 main                 [clean]
  backend:             main                 [modified]
  shared-lib:          main                 [clean]

feature-payment
----------------------------------------  
  app:                 feature-payment      [clean]
  backend:             feature-payment      [clean]
  shared-lib:          feature-payment      [modified]
```

### `workspace foreach <command>`
Execute a command in all repositories of the current workspace (similar to `git submodule foreach`).

```bash
cd worktrees/main
../../workspace foreach git status -s       # Check status in all repos
../../workspace foreach make test           # Run tests in all repos
../../workspace foreach git log --oneline -5 # Show recent commits
../../workspace foreach --quiet git pull    # Update all repos quietly

# Using environment variables (available in each command):
../../workspace foreach 'echo $name is at $path'
../../workspace foreach 'git -C $toplevel/$path log --oneline -1'
```

**Behavior:**
- Must be run from within a workspace directory
- Executes command in each repository sequentially
- Shows clear headers for each repository (unless `--quiet` is used)
- Stops on first error (due to `set -e`)
- Provides environment variables: `$name`, `$path`, `$displaypath`, `$toplevel`

### `workspace list`
List all available workspaces.

```bash
./workspace list
```

**Output:**
```
Available Workspaces
==================
  main
  feature-payment
  release-v2.0
```

### `workspace clean <branch>`
Remove a workspace and all its repositories.

```bash
./workspace clean feature-payment
```

**Behavior:**
- Prompts for confirmation before deletion
- Removes entire workspace directory
- Cannot be undone - use with caution

### `workspace repair <workspace> <repository>`
Repair broken or corrupted repositories in a workspace.

```bash
./workspace repair main app
./workspace repair feature-x backend
```

**Behavior:**
- Detects and fixes various broken repository states:
  - Uninitialized repositories (cloned but no commits)
  - Corrupted worktree references  
  - Standalone repositories that should be worktrees
  - Invalid HEAD references
- Converts standalone repos to worktree architecture when needed
- Recreates repositories from central cache if necessary
- Handles timeout issues during fetch operations

**Common use cases:**
- Repository shows `[uninitialized]` or `[broken]` in status
- After network interruptions during repository setup
- When converting legacy setups to worktree architecture
- Fixing repositories after git operations fail

### `workspace config <subcommand>`
Manage per-workspace repository configurations.

```bash
# Set workspace-specific configuration
./workspace config set feature-x https://github.com/org/app.git feature-branch v2.0

# Show configuration for a workspace
./workspace config show feature-x

# Template-based configuration (current method)
./workspace config set-default https://github.com/org/repo.git

# Set default configuration for all workspaces
./workspace config set-default https://github.com/org/shared.git main
```

**Subcommands:**
- `set <workspace> <repo-name> <branch> [ref]` - Override branch for specific workspace
- `show [workspace]` - Display effective configuration  
- `set-default <url> [branch] [ref]` - Add repository to template
- `help` - Show detailed usage and examples

### `workspace help`
Show help information and usage examples.

### `workspace install-completion [shell]`
Install shell completion for bash or zsh. Auto-detects shell if not specified.

The workspace script includes shell completion support for both bash and zsh to enhance your command-line experience.

#### Features

- Command name completion (switch, sync, status, etc.)
- Workspace name completion for relevant commands
- Repository name completion for config commands
- Branch name suggestions
- Config subcommand completion
- Smart context-aware suggestions

#### Installation

##### Automatic Installation (Recommended)

```bash
# Auto-detect shell and install
./workspace install-completion

# Or specify shell explicitly
./workspace install-completion bash
./workspace install-completion zsh
```

This installs completions to standard XDG directories:
- **Bash**: `~/.local/share/bash-completion/completions/workspace`
- **Zsh**: `~/.local/share/zsh/site-functions/_workspace`

These locations work on all systems including Nix/NixOS where shell init files are read-only.

##### Manual Installation

**Bash:**
```bash
mkdir -p ~/.local/share/bash-completion/completions
cp workspace-completion.bash ~/.local/share/bash-completion/completions/workspace
```

**Zsh:**
```bash
mkdir -p ~/.local/share/zsh/site-functions
cp workspace-completion.zsh ~/.local/share/zsh/site-functions/_workspace
```

**Note**: If zsh completions don't work, ensure the XDG directory is in your fpath:
```bash
# Add to .zshrc:
fpath=($HOME/.local/share/zsh/site-functions $fpath)
autoload -Uz compinit && compinit

# To customize or remove the "Completing" prefix in zsh:
zstyle ':completion:*:descriptions' format '%B%d:%b'  # Show just description with colon
# Or remove descriptions entirely:
zstyle ':completion:*:descriptions' format ''
```

##### Home Manager Integration

For Nix/NixOS users managing their dotfiles with Home Manager:

**Zsh:**
```nix
# Ensure XDG directories are in fpath
programs.zsh.initContent = ''
  fpath=($HOME/.local/share/zsh/site-functions $fpath)
  autoload -Uz compinit && compinit
'';
```

**Bash:**
Most bash configurations automatically load from `~/.local/share/bash-completion/completions/`, but if needed:
```nix
programs.bash.initExtra = ''
  [[ -f ~/.local/share/bash-completion/completions/workspace ]] && \
    source ~/.local/share/bash-completion/completions/workspace
'';
```

#### Usage Examples

Once installed, you can use tab completion with the workspace command:

```bash
# Complete command names
workspace <TAB>
# Shows: switch sync status foreach list clean config repair install-completion help

# Complete workspace names for switch
workspace switch <TAB>
# Shows: main develop feature-x (existing workspaces)

# Complete workspace names for sync
workspace sync <TAB>
# Shows: main develop feature-x (existing workspaces)

# Complete config subcommands
workspace config <TAB>
# Shows: set-default set show help

# Complete repository names for config set
workspace config set feature-x <TAB>
# Shows: core utils docs (repository names from template)

# Complete commands for foreach
workspace foreach <TAB>
# Shows: existing workspaces or commands

# Chain completions
workspace config set feature-x core <TAB>
# Shows: main master develop (branch suggestions)
```

#### Completion Details

**Bash Completion:**
- Command completion for all workspace subcommands
- Workspace name completion where applicable
- Repository name extraction from git config
- Common branch name suggestions
- Config subcommand completion
- Contextual hints showing what values are expected

**Zsh Completion:**
- All bash features plus:
- Descriptive help for each command
- Better integration with zsh's completion system
- Alternative suggestions (e.g., existing workspaces vs branch names)
- Context-aware command completion for `foreach`

#### Testing Completions

To verify completion is working correctly:

```bash
# Run the test script
./test-completion.sh
```

This will check that:
- Completion files exist
- They load without errors
- Functions are properly defined
- All commands are documented

#### Troubleshooting

**Bash:**
If completion doesn't work:
1. Ensure bash-completion package is installed
2. Check that the completion file is sourced: `complete -p workspace`
3. Verify the path in your .bashrc is correct

**Zsh:**
If completion doesn't work:
1. Ensure compinit is called in your .zshrc
2. Check fpath includes the completion directory: `echo $fpath`
3. Try running `compinit` manually
4. Remove cached completions: `rm ~/.zcompdump*`

#### Contributing

To modify the completion scripts:
- `workspace-completion.bash` - Bash completion logic
- `workspace-completion.zsh` - Zsh completion logic
- Update both scripts when adding new commands or options
- Test changes with `test-completion.sh`

## Use Cases & Workflows

### 1. **Feature Development**
```bash
# Start new feature
./workspace switch feature-user-auth
cd worktrees/feature-user-auth

# Work across repositories
../../workspace foreach git checkout -b feature-user-auth
# ... make changes in multiple repos ...
../../workspace foreach git commit -m "Add user authentication"

# Keep feature branch in sync
../../workspace sync
```

### 2. **Release Management**
```bash
# Create release workspace
./workspace switch release-v2.0
cd worktrees/release-v2.0

# Prepare release
../../workspace foreach git checkout -b release-v2.0
../../workspace foreach ./scripts/bump-version.sh 2.0.0
../../workspace foreach git commit -m "Bump version to 2.0.0"

# Final testing
../../workspace foreach make test
../../workspace foreach make build
```

### 3. **Dependency Management**
Update template configuration to pin a new version:
```bash
# Update template with new version
./workspace config set-default https://github.com/vendor/sdk.git main v1.2.0
```

Then recreate workspaces:
```bash
./workspace clean main
./workspace switch main
```

### 4. **Parallel Development**
```bash
# Multiple developers, multiple features
./workspace switch feature-payments
./workspace switch feature-notifications  
./workspace switch hotfix-security

# Each workspace is completely isolated
./workspace status  # See all active workspaces
```

### 5. **Per-Workspace Repository Configuration**
```bash
# Scenario: Testing a feature that spans specific repos
# while keeping others on stable branches

# Set up main workspace with stable versions
./workspace config set main https://github.com/org/app.git main
./workspace config set main https://github.com/org/backend.git main  
./workspace config set main https://github.com/org/database.git main v3.0.0

# Set up feature workspace with mixed branches
./workspace config set feature-api https://github.com/org/app.git main
./workspace config set feature-api https://github.com/org/backend.git feature-api-v2
./workspace config set feature-api https://github.com/org/database.git main v3.0.0

# Each workspace maintains its own configuration
./workspace switch main          # Uses main branches
./workspace switch feature-api   # Uses feature branch for backend only
```

### 6. **CI/CD Integration**
```bash
#!/bin/bash
# ci-build.sh
set -e

# Switch to workspace for current branch
./workspace switch "$GITHUB_REF_NAME"
cd "worktrees/$GITHUB_REF_NAME"

# Build everything
../../workspace foreach make clean build test
```

## Expected Behaviors

### Repository Cloning
- **Branch exists remotely**: Creates worktree for specific branch directly
- **Branch doesn't exist**: Creates worktree from default branch, then creates local branch
- **Pinned repository**: Always creates worktree for specified branch, then checks out ref

### Synchronization
- **Branch-tracking repos**: Updated with `git pull --ff-only`
- **Pinned repos**: Skipped with warning message
- **Missing repos**: Warning logged, operation continues
- **Merge conflicts**: Command fails, manual resolution required

### Error Handling
- **Missing config file**: Creates empty workspace
- **Invalid git URL**: Worktree creation fails, error reported
- **Network issues**: Operation fails with git error message
- **Permission issues**: Fails with appropriate error

### Workspace Isolation
- Each workspace is completely independent
- No shared state between workspaces
- Safe to delete workspaces without affecting others
- Can have different versions/branches per workspace

## Advanced Patterns

### Mixed Repository Types
```bash
# Template configuration (replaces workspace.conf)
./workspace config set-default git@github.com:org/frontend.git
./workspace config set-default git@github.com:org/backend.git

# Shared library - always use main branch
./workspace config set-default git@github.com:org/shared-components.git main

# Third-party - pinned version
./workspace config set-default https://github.com/vendor/ui-kit.git v2 v2.1.0
```

### Branch-Specific Dependencies
Create different configs for different environments:
```bash
# workspace-dev.conf - development dependencies
https://github.com/vendor/sdk.git main v1.0.0-beta

# workspace-prod.conf - production dependencies  
https://github.com/vendor/sdk.git main v1.0.0
```

Use with:
```bash
CONFIG_FILE=workspace-dev.conf ./workspace switch develop
```

## Migration from Git Submodules

### 1. Extract Current Configuration
```bash
# Extract repository URLs from .gitmodules and set up templates
git config -f .gitmodules --get-regexp '^submodule\..*\.url$' | \
  awk '{ print $2 }' | while read url; do
    ./workspace config set-default "$url"
  done
```

### 2. Remove Submodules
```bash
# Remove submodule configuration
git rm .gitmodules
rm -rf .git/modules/

# Remove submodule directories (they'll be recreated as worktrees)
git submodule deinit --all
git submodule | cut -d' ' -f3 | xargs rm -rf

# Commit the changes  
git add -A
git commit -m "Migrate from submodules to wt-super"
```

### 3. Start Using wt-super
```bash
./workspace switch main
# Your repositories are now managed by wt-super!
```

## Comparison with Alternatives

| Feature | Git Submodules | wt-super | Git Worktree |
|---------|---------------|----------|--------------|
| **Learning Curve** | High | Low | Medium |
| **Multi-repo Support** | Yes | Yes | Single repo only |
| **Version Pinning** | Complex | Simple | N/A |
| **Isolated Workspaces** | No | Yes | Yes |
| **Clean History** | No (pointer commits) | Yes | Yes |
| **Branch Coordination** | Manual | Automatic | Manual |
| **Setup Complexity** | High | Low | Medium |

## Troubleshooting

### Common Issues

**"Repository not found" during worktree creation:**
- Check URL format and access permissions
- Verify SSH keys are configured for private repositories

**"Workspace not found" for sync/exec:**
-Run commands from correct directory or specify workspace name

**"Branch doesn't exist" warnings:**
- Create branches in upstream repositories first
- Or let wt-super create local branches automatically

**Permission denied errors:**
- Check file permissions on workspace script: `chmod +x workspace`
- Verify git repository access permissions

### Debug Mode
Run with verbose output:
```bash
set -x
./workspace switch feature-test
set +x
```

## Development & Testing

See [README_TESTING.md](README_TESTING.md) for comprehensive test suite documentation.

**Quick test run:**
```bash
uv run --extra test pytest  # Runs all tests (100+) with enhanced output
```

**Using Nix:**
If you have Nix available (common on NixOS, but works anywhere Nix is installed), you can use this approach to avoid UV's Python download issues:
```bash
# Run tests with Nix-provided Python and parallel execution
nix-shell -p python312 uv bash git --run "UV_PYTHON_DOWNLOADS=never uv run --extra test pytest -n auto"

# Or for non-parallel execution
nix-shell -p python312 uv bash git --run "UV_PYTHON_DOWNLOADS=never uv run --extra test pytest"
```

This forces UV to use Nix's properly-linked Python while maintaining UV's dependency management.

**Enhanced Test Output:**
This project uses `pytest-rich` for improved test output featuring:
- **Rich color formatting** with syntax highlighting
- **Beautiful tracebacks** with enhanced error details
- **Real-time progress display** with visual indicators
- **Improved readability** through rich text formatting

**Test Categories:**
- **Fast tests** (default): Local functionality tests
- **Slow tests**: Large-scale operations, marked with `@pytest.mark.slow`
- **Network tests**: Remote repository access, marked with `@pytest.mark.network`

**Git Credentials for Network Tests:**
Network tests inherit your global git configuration, including credential helpers:
```bash
# Configure git credential helper (if not already done)
git config --global credential.helper store

# Network tests will use your configured credentials
uv run --extra test pytest -m "network"

# Test credential configuration inheritance
uv run --extra test pytest -k "test_git_config_inheritance" -v
```

**Running specific test categories:**
```bash
# Fast tests only (recommended for development) - DEFAULT
uv run --extra test pytest -m "not slow and not network"

# All tests except network (good for CI)
uv run --extra test pytest -m "not network"

# Only slow tests
uv run --extra test pytest -m "slow"

# Only network tests (may prompt for credentials - requires internet)
uv run --extra test pytest -m "network"

# Nix equivalents (add parallel execution with -n auto)
nix-shell -p python312 uv bash git --run "UV_PYTHON_DOWNLOADS=never uv run --extra test pytest -m 'not network' -n auto"

# All tests including network (use with caution - may prompt for credentials)
uv run --extra test pytest
```

**Parallel Test Execution:**
Tests can be run in parallel for significant speedup (3-4x faster):
```bash
# Parallel execution with auto-detected CPU cores
uv run --extra test pytest -n auto -m "not network"

# Parallel with specific number of workers
uv run --extra test pytest -n 4 -m "not network"

# Serial execution (if needed for debugging)
uv run --extra test pytest -m "not network"
```

**Additional Options:**
```bash
# Disable enhanced output (use standard pytest format)
uv run --extra test pytest -p no:rich

# Combine parallel + specific test selection
uv run --extra test pytest -n auto -k "test_init"
```

## Contributing

1. Fork the repository
2. Create your feature branch: `git checkout -b feature-amazing`
3. Add tests for new functionality
4. Run the test suite: `uv run --extra test pytest`
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature-amazing`
7. Open a Pull Request

## License

MIT License - see LICENSE file for details.

## Why "wt-super"?

**wt** = **w**ork**t**ree (the Git feature we conceptually build upon)  
**super** = **super**-project (like a Git superproject, but simpler)

The name reflects the tool's purpose: providing superproject-like functionality using worktree concepts, but with much greater simplicity than Git submodules.