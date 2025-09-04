# Prompt for Next Testing Session

## Use this prompt to start your next session with optimal context:

---

I need to expand test coverage for the git-worktree-superproject tool that manages multiple repositories using git worktrees instead of submodules. The tool has been recently refactored to support per-workspace repository configurations.

**Repository:** `/home/tim/src/git-worktree-superproject`
**Branch:** `per-workspace-config`

## Current State
- Core refactoring is complete and functional
- Basic test structure exists in `test/` directory
- New test file `test_per_workspace_config.py` covers new config features
- The tool now supports both legacy `workspace.conf` and new git-config-based per-workspace configurations

## Key Components to Test

### Core Commands (need comprehensive testing):
1. **`workspace switch [branch]`** - Creates/switches workspaces as git worktrees
2. **`workspace sync [branch]`** - Updates repositories in workspace
3. **`workspace status`** - Shows all workspaces and repo states
4. **`workspace foreach <cmd>`** - Executes commands in all repos
5. **`workspace list`** - Lists available workspaces
6. **`workspace clean <branch>`** - Removes workspace and its worktree

### Config Commands (partially tested):
- `workspace config set <workspace> <url> [branch] [ref]`
- `workspace config show [workspace]`
- `workspace config import <workspace> [file]`
- `workspace config set-default <url> [branch] [ref]`

## Critical Test Scenarios Needed

### 1. Workspace Lifecycle Tests
- Create workspace → add repos → modify → sync → clean
- Multiple concurrent workspaces
- Workspace with same name as existing branch
- Recovery from interrupted operations

### 2. Repository State Management
- Branch tracking (following workspace branch)
- Pinned refs (detached HEAD at specific commit/tag)
- Mixed configurations (some tracking, some pinned)
- Missing/corrupted repositories
- Network failures during clone/fetch

### 3. Configuration Inheritance
- Workspace-specific overrides default
- Default overrides legacy file
- Empty configurations at various levels
- Migration from workspace.conf to git config
- Config conflicts and resolution

### 4. Edge Cases
- Repositories with spaces in names/paths
- Very large repositories
- Submodules within managed repositories
- Symbolic links in repository paths
- Permission issues
- Concurrent operations on same workspace

### 5. Integration Workflows
From README.md use cases:
- Feature development across multiple repos
- Release management with version pinning
- Parallel development on different features
- CI/CD integration scenarios
- Team collaboration with different workspace configs

## Test Framework Setup
- Using pytest with fixtures in `conftest.py`
- Class-scoped fixtures for expensive setup (base_git_repos)
- Test parallelization with pytest-xdist
- Existing fixtures: temp_workspace, git_repos, run_workspace

## Documentation References
- **README.md** - Full command documentation and use cases
- **PER-WORKSPACE-CONFIG-DESIGN.md** - Architecture and design decisions
- **MIGRATION-GUIDE.md** - Migration scenarios to test

## Testing Goals
1. Achieve >90% code coverage for workspace script
2. Validate all documented use cases work correctly
3. Ensure backward compatibility with workspace.conf
4. Verify per-workspace configurations are isolated
5. Test error handling and recovery scenarios
6. Validate CI/CD integration patterns

## First Priority
Focus on testing the core workspace commands (switch, sync, status, foreach, clean) as these are essential for basic functionality and most likely to be used immediately. The config commands have basic coverage but the core commands need comprehensive testing.

Please analyze the current test coverage, identify gaps, and implement comprehensive tests for the workspace commands following the patterns established in the existing test files.

---

## Additional Context Files to Review:
- `workspace` - The main script to test
- `test/conftest.py` - Test fixtures and utilities
- `test/test_workspace.py` - Existing workspace tests (may need updates)
- `test/test_config.py` - Configuration parsing tests
- `test/test_per_workspace_config.py` - New config system tests