# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## ⚠️ CRITICAL ARCHITECTURAL ISSUE

**The project has a fundamental naming/implementation mismatch that requires major refactoring.**

### Current Problem

The project is named `wt-super` (worktree-super) and claims to use "Git Worktrees" in its title and documentation, but **the implementation uses `git clone` instead of `git worktree` commands**. This is misleading and defeats the core benefits of actual git worktrees.

**Current Implementation:**
- Uses `git clone` to create completely separate repository copies
- Each workspace has independent `.git` directories
- No shared git objects or history
- High disk usage due to duplicate git data
- No actual `git worktree` commands anywhere in the codebase

**Promised Implementation (from name/docs):**
- Should use `git worktree add` to create linked working trees
- Should share `.git` database between all worktrees
- Should provide disk space efficiency
- Should allow seamless switching between branches across worktrees

### Proposed Refactoring Plan

#### Phase 1: Analysis and Design (1-2 days)
1. **Document Current Architecture**
   - Map all current `git clone` operations
   - Identify workspace creation/management patterns
   - Analyze configuration parsing logic

2. **Design True Worktree Architecture**
   - Multi-repository worktree coordination strategy
   - Shared vs. independent worktree management
   - Configuration format implications
   - Command interface changes needed

3. **Compatibility Planning**
   - Migration path from clone-based to worktree-based
   - Backward compatibility considerations
   - Configuration file evolution

#### Phase 2: Core Implementation Refactor (3-5 days)
1. **Replace Clone Operations with Worktree Operations**
   ```bash
   # Current: git clone -b "$branch" "$url" "$repo_path"
   # New: git worktree add "$worktree_path" "$branch"
   ```

2. **Implement Multi-Repository Worktree Coordinator**
   - Central repository management
   - Per-repo worktree lifecycle
   - Branch synchronization across repos

3. **Update Core Commands**
   - `switch`: Use `git worktree add` instead of clone
   - `sync`: Coordinate pulls across linked worktrees
   - `clean`/`remove`: Use `git worktree remove` 
   - `status`: Show worktree states, not clone states

#### Phase 3: Configuration and Tooling (1-2 days)
1. **Repository Initialization Strategy**
   - Initial clone vs. worktree creation
   - Central repository location management
   - Multi-repo coordination patterns

2. **Enhanced Configuration**
   - Support for worktree-specific options
   - Central repository path configuration
   - Worktree naming and organization

#### Phase 4: Testing Overhaul (2-3 days)
1. **Replace All Clone-Based Tests**
   - Remove tests that verify separate `.git` directories
   - Add tests that verify shared git databases
   - Test worktree-specific operations

2. **Add Worktree-Specific Test Coverage**
   ```python
   def test_worktrees_share_git_database():
       """Verify worktrees share git objects and refs."""
   
   def test_worktree_add_remove_operations():
       """Test git worktree add/remove directly."""
   
   def test_multi_repo_worktree_coordination():
       """Test coordinated worktree operations across repos."""
   ```

3. **Performance and Disk Usage Tests**
   - Verify disk space efficiency vs. clone approach
   - Test worktree operation performance
   - Validate shared git database benefits

#### Phase 5: Documentation and Migration (1 day)
1. **Update All Documentation**
   - Accurate description of worktree usage
   - Migration guide from clone-based installations
   - Benefits comparison: worktrees vs. clones vs. submodules

2. **Migration Tooling**
   - Script to convert clone-based workspaces to worktrees
   - Validation tools for proper worktree setup
   - Cleanup utilities for old clone-based workspaces

### Implementation Strategy Details

#### Multi-Repository Worktree Management
Since git worktrees work per-repository, the tool needs to coordinate multiple repositories each with their own worktrees:

```bash
# Structure should become:
project/
├── workspace.conf
├── repos/                    # Central repositories
│   ├── repo-a/              # Main repo-a clone
│   │   ├── .git/
│   │   └── worktrees/       # Git's worktree metadata
│   ├── repo-b/              # Main repo-b clone  
│   └── repo-c/              # Main repo-c clone
└── worktrees/               # Linked worktrees
    ├── main/
    │   ├── repo-a/          # Worktree linked to repos/repo-a
    │   ├── repo-b/          # Worktree linked to repos/repo-b
    │   └── repo-c/          # Worktree linked to repos/repo-c
    └── feature-branch/
        ├── repo-a/          # Different branch, same repo
        ├── repo-b/
        └── repo-c/
```

#### Key Implementation Changes Needed

1. **Repository Initialization**
   ```bash
   # Instead of: git clone "$url" "$path"
   ensure_central_repo "$url" "$repo_name"
   git -C "repos/$repo_name" worktree add "../../worktrees/$workspace/$repo_name" "$branch"
   ```

2. **Workspace Switching**
   ```bash
   # Instead of: cd to cloned directory
   # Git worktrees allow seamless branch switching across linked trees
   ```

3. **Cleanup Operations**
   ```bash
   # Instead of: rm -rf "$workspace_dir"
   for repo in $(list_repos); do
       git -C "repos/$repo" worktree remove "../../worktrees/$workspace/$repo"
   done
   ```

### Testing Requirements

#### New Test Categories Needed
1. **Worktree Integrity Tests**
   - Verify shared `.git` database
   - Test git object sharing between worktrees
   - Validate worktree metadata consistency

2. **Multi-Repository Coordination Tests**
   - Synchronized branch creation across repos
   - Coordinated worktree lifecycle management
   - Cross-repository operation atomicity

3. **Performance and Efficiency Tests**
   - Disk space usage comparison (worktrees vs clones)
   - Operation speed benchmarks
   - Git database efficiency validation

4. **Migration and Compatibility Tests**
   - Clone-to-worktree migration scenarios
   - Configuration backward compatibility
   - Cleanup and validation procedures

### Risk Assessment

#### High Risk Areas
1. **Data Loss During Migration**
   - Existing clone-based workspaces need careful migration
   - Git history preservation during conversion
   - Uncommitted changes protection

2. **Multi-Repository Complexity**
   - Coordinating worktrees across multiple repositories
   - Handling repository-specific failures gracefully
   - Maintaining consistency across repo states

3. **Git Worktree Limitations**
   - Some git operations don't work well with worktrees
   - Platform-specific worktree behavior differences
   - Nested worktree scenarios

#### Mitigation Strategies
1. **Incremental Migration Path**
   - Support both clone and worktree modes during transition
   - Extensive testing with real-world repositories
   - Rollback procedures for failed migrations

2. **Comprehensive Testing**
   - Test with various git versions and platforms
   - Stress testing with many repositories and worktrees
   - Performance regression testing

### Success Criteria

1. **Functional Accuracy**
   - All operations use actual `git worktree` commands
   - Shared git databases verified across worktrees
   - Disk space usage significantly reduced vs. clones

2. **Performance Improvements**
   - Faster workspace switching (no cloning required)
   - Reduced disk usage (shared git objects)
   - Improved sync performance (shared git database)

3. **User Experience**
   - Seamless migration from clone-based installations
   - Maintained command interface compatibility
   - Clear documentation of benefits and usage

4. **Test Coverage**
   - 100% replacement of clone-based tests with worktree tests
   - New worktree-specific functionality coverage
   - Performance and efficiency validation tests

## Development Commands

### Running Tests
```bash
# Run all tests (excludes network tests by default)
uv run --extra test pytest

# Run with parallel execution (3-4x faster)
uv run --extra test pytest -n auto

# Run specific test categories
uv run --extra test pytest -m "not slow and not network"  # Fast tests only
uv run --extra test pytest -m "slow"                       # Slow tests only
uv run --extra test pytest -m "network"                    # Network tests (requires internet)

# Run with coverage
uv run --extra test pytest --cov=.

# Run specific test file
uv run --extra test pytest test/test_workspace.py

# Verbose output
uv run --extra test pytest -v
```

### Using Nix
If you have Nix available (common on NixOS, but works anywhere Nix is installed), you can use this approach to avoid UV's Python download issues:
```bash
# Run tests with Nix-provided Python and parallel execution
nix-shell -p python312 uv bash git --run "UV_PYTHON_DOWNLOADS=never uv run --extra test pytest -n auto"

# Or for non-parallel execution
nix-shell -p python312 uv bash git --run "UV_PYTHON_DOWNLOADS=never uv run --extra test pytest"
```

This forces UV to use Nix's properly-linked Python while maintaining UV's dependency management.

### Test Structure
- **test/conftest.py**: Shared fixtures (temp_workspace, git_repos, run_workspace)
- **test/test_workspace.py**: Core functionality tests
- **test/test_workspace_advanced.py**: Advanced features and edge cases
- **test/test_config.py**: Configuration parsing tests
- **test/test_superproject_*.py**: Superproject-specific scenarios
- **test/fixtures/**: Sample configuration files

### Key Testing Considerations
- Tests use pytest-result-bar for enhanced output formatting
- Tests can run in parallel with pytest-xdist (`-n auto`)
- Network tests are marked and excluded by default
- Slow tests are marked for selective execution
- Tests create isolated temporary directories for each run

## Important Implementation Details

### Current Architecture (TO BE REPLACED)
- Clone operations check if remote branch exists before cloning
- Falls back to default branch if specified branch doesn't exist
- Uses `--ff-only` for pulls to prevent merge conflicts
- Pinned repositories remain in detached HEAD state

### Target Worktree Architecture (TO BE IMPLEMENTED)
- Central repositories with worktree coordination
- Shared git databases for space efficiency
- `git worktree add/remove` operations
- Multi-repository worktree lifecycle management

### Environment Variables in foreach
- `$name`: Repository name
- `$path`: Repository path relative to workspace
- `$displaypath`: Same as $path
- `$toplevel`: Workspace root directory

## Workspace Commands (Current - TO BE REFACTORED)
1. **switch [branch]**: Switch to workspace, create if needed
2. **sync [branch]**: Update repos (skips pinned ones)
3. **status**: Show all workspaces and their repo states
4. **foreach <command>**: Execute command in all repos
5. **list**: Show available workspaces
6. **clean <branch>**: Remove workspace (with confirmation)

## Important Instructions

### Development Priorities
1. **DO NOT ADD NEW FEATURES** until the worktree refactoring is complete
2. **ALL CHANGES** should move toward true worktree implementation
3. **MAINTAIN BACKWARD COMPATIBILITY** during transition period
4. **COMPREHENSIVE TESTING** is required for all worktree operations

### Code Quality Standards
- Never create files unless absolutely necessary for achieving goals
- Always prefer editing existing files to creating new ones
- Never proactively create documentation files unless explicitly requested
- Follow existing code patterns and conventions
- Maintain test coverage throughout refactoring

### Commit Message Guidelines
- Never include Claude or Anthropic attribution in git commit messages
- Do not add "Generated with Claude Code" or "Co-Authored-By: Claude" lines
- Focus commit messages on the technical changes and their purpose

### Working Directory Best Practices  
- Always explicitly state the current working directory (PWD) when running commands with relative paths
- Use absolute paths when possible to avoid confusion
- When using relative paths, confirm PWD before executing commands

## Migration Analysis (Phase 1 COMPLETED)

### Current Clone Operations Identified
- **Line 51**: `git clone -b "$repo_branch" "$url" "$repo_path" --quiet`  
- **Line 53**: `git clone "$url" "$repo_path" --quiet` (fallback)
- **Line 184**: `rm -rf "$workspace_dir"` (should use `git worktree remove`)
- **Line 114**: Status checks for independent `.git` directories

### Compatibility Strategy  
1. **Backward Compatible Migration**
   - Detect existing clone-based workspaces
   - Preserve user data during transition
   - Support both modes during migration period

2. **Configuration Preservation**
   - Existing `workspace.conf` format unchanged
   - No breaking changes to user workflows
   - Transparent migration process

3. **Data Safety Measures**
   - Validate git history preservation
   - Backup existing workspaces before conversion
   - Rollback procedures for failed migrations

### Implementation Roadmap
Phase 1 ✓: Analysis and planning completed
Phase 2: Replace core clone operations with worktree operations
Phase 3: Add central repository management and enhanced configuration  
Phase 4: Comprehensive test suite overhaul
Phase 5: Documentation updates and migration tooling

## Phase Completion Protocol
**IMPORTANT**: After completing each Phase and committing the changes, STOP and return control to the user before proceeding to the next Phase. The user will /clear the conversation history between phases to manage context efficiently.