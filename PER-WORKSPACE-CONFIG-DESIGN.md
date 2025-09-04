# Per-Workspace Configuration Design & Refactoring Plan

## Executive Summary

This document outlines the refactoring of `git-worktree-superproject` to support **per-workspace repository configurations**. Currently, all workspaces share a single `workspace.conf` file. The new design allows each workspace to have independent repository specifications, tracked in git history.

## Problem Statement

**Current Limitation**: "There is not currently any way to track per-worktree repo configuration. We only have a single workspace.conf."

When working with multiple workspaces (e.g., `main`, `feature-x`, `hotfix`), each workspace is forced to use identical repository specifications. This prevents scenarios like:
- Testing a feature branch across specific repos while keeping others on main
- Having a stable workspace with pinned versions alongside a development workspace
- Different team members working with different repository subsets

## Architectural Design

### Core Concept

Transform workspace directories from plain directories to **git worktrees of the superproject**, enabling:
1. Per-workspace configuration via git's worktree-specific config
2. Version-controlled workspace configurations
3. Configuration inheritance (workspace-specific → superproject defaults)

### Directory Structure Evolution

**Current Structure:**
```
superproject/
├── workspace.conf                 # Single shared config
├── workspace                      # Management script
├── repos/                         # Central bare repositories
└── worktrees/                     # Plain directories
    └── dev/                       # NOT a git worktree
        └── [repo worktrees]       # Git worktrees of repos/
```

**New Structure:**
```
superproject/
├── .git/
│   ├── config                     # Default workspace config
│   └── worktrees/
│       └── dev/
│           └── config.worktree    # Dev workspace-specific config
├── workspace.conf                 # DEPRECATED (migration only)
├── workspace                      # Updated management script
├── repos/                         # Central bare repositories (unchanged)
└── worktrees/
    └── dev/                       # NOW a git worktree of superproject
        ├── .git                   # File pointing to ../../.git/worktrees/dev
        └── [repo worktrees]       # Nested worktrees (git supports this)
```

### Configuration Storage

**Git Config Format** (replaces workspace.conf):
```ini
# In .git/config (defaults for all workspaces)
[workspace]
    repo = projectA https://github.com/org/projectA.git main
    repo = projectB https://github.com/org/projectB.git main

# In .git/worktrees/dev/config.worktree (workspace-specific)
[workspace]
    repo = projectA https://github.com/org/projectA.git feature-branch
    repo = projectC https://github.com/org/projectC.git main v2.0.0
[workspace "metadata"]
    created = 2024-01-15
    purpose = "Feature development for XYZ"
    owner = "developer-name"
```

## Implementation Changes

### Workspace Script Modifications

#### 1. **Initialization (new)**
```bash
# One-time setup when first run
init_superproject() {
    if [[ ! -d .git ]]; then
        git init
        git config extensions.worktreeConfig true
        echo "/repos/" >> .gitignore
        echo "/worktrees/" >> .gitignore
        git add .
        git commit -m "Initialize workspace superproject"
    fi
}
```

#### 2. **load_repos() - Multi-source configuration**
```bash
load_repos() {
    local workspace="${1:-main}"
    
    # Priority 1: Worktree-specific git config
    if [[ -d "$WORKTREES_DIR/$workspace/.git" ]]; then
        local config=$(cd "$WORKTREES_DIR/$workspace" && \
                      git config --worktree --get-all workspace.repo 2>/dev/null)
        [[ -n "$config" ]] && { echo "$config"; return; }
    fi
    
    # Priority 2: Superproject default config
    local default_config=$(git config --get-all workspace.repo 2>/dev/null)
    [[ -n "$default_config" ]] && { echo "$default_config"; return; }
    
    # Priority 3: Legacy workspace.conf (for migration)
    [[ -f "$CONFIG_FILE" ]] && load_repos_from_file
}
```

#### 3. **switch_workspace() - Create superproject worktrees**
```bash
switch_workspace() {
    local branch="${1:-main}"
    local workspace_dir="$WORKTREES_DIR/$branch"
    
    # Create superproject worktree (not just directory)
    if [[ ! -d "$workspace_dir/.git" ]]; then
        echo "Creating workspace as superproject worktree..."
        git worktree add "$workspace_dir" -b "workspace/$branch" 2>/dev/null || \
        git worktree add "$workspace_dir" "workspace/$branch"
    fi
    
    # Continue with existing repo setup logic
    # ... (rest remains largely the same)
}
```

#### 4. **New config management commands**
```bash
# workspace config set <workspace> <repo-name> <url> [branch] [ref]
config_set() {
    local workspace="$1"
    local repo_name="$2"
    local url="$3"
    local branch="${4:-main}"
    local ref="${5:-}"
    
    cd "$WORKTREES_DIR/$workspace"
    git config --worktree "workspace.repo.$repo_name" "$url $branch $ref"
}

# workspace config show [workspace]
config_show() {
    local workspace="${1:-main}"
    echo "Configuration for workspace: $workspace"
    
    if [[ -d "$WORKTREES_DIR/$workspace/.git" ]]; then
        cd "$WORKTREES_DIR/$workspace"
        git config --worktree --get-regexp "^workspace\."
    fi
}

# workspace config import <workspace> < workspace.conf
config_import() {
    local workspace="$1"
    while read -r line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ -z "${line// }" ]] && continue
        
        read -ra parts <<< "$line"
        local url="${parts[0]}"
        local branch="${parts[1]:-main}"
        local ref="${parts[2]:-}"
        local name=$(basename "${url%.git}")
        
        config_set "$workspace" "$name" "$url" "$branch" "$ref"
    done
}
```

### Test Suite Adaptation

The existing test suite is well-structured and mostly reusable. Key changes needed:

#### Test Categories & Reusability

1. **Fully Reusable Tests** (~70% of suite):
   - `test_workspace.py` - Core workspace operations remain the same
   - `test_superproject_edge_cases.py` - Edge cases still apply
   - `test_missing_coverage.py` - Coverage requirements unchanged
   - Most fixture logic in `conftest.py`

2. **Tests Needing Minor Updates** (~20%):
   - `test_config.py` - Add tests for git config in addition to file config
   - `test_workspace_advanced.py` - Add workspace isolation tests
   
3. **New Tests Required** (~10%):
   - Git worktree initialization tests
   - Config inheritance tests (worktree → default → file)
   - Migration tests (workspace.conf → git config)
   - Metadata storage/retrieval tests

#### Specific Test Adaptations

**conftest.py fixture updates:**
```python
@pytest.fixture
def temp_workspace_git_enabled(tmp_path):
    """Create temp workspace with git initialization."""
    workspace_dir = tmp_path / "test_workspace"
    workspace_dir.mkdir()
    os.chdir(workspace_dir)
    
    # Initialize as git repo
    subprocess.run(["git", "init"], check=True)
    subprocess.run(["git", "config", "extensions.worktreeConfig", "true"], check=True)
    
    # Create initial commit
    (workspace_dir / ".gitignore").write_text("worktrees/\nrepos/\n")
    subprocess.run(["git", "add", "."], check=True)
    subprocess.run(["git", "commit", "-m", "Initial"], check=True)
    
    yield workspace_dir
```

**test_config.py additions:**
```python
class TestGitConfigParsing:
    """Test git-config based configuration."""
    
    def test_worktree_specific_config(self, temp_workspace_git_enabled, run_workspace):
        """Test workspace-specific git config overrides."""
        # Set default config
        subprocess.run(["git", "config", "--add", "workspace.repo", 
                       "projectA http://example.com/a.git main"], check=True)
        
        # Create workspace
        run_workspace("switch", "feature")
        
        # Set worktree-specific config
        subprocess.run(["git", "config", "--worktree", "--add", "workspace.repo",
                       "projectA http://example.com/a.git feature-branch"],
                      cwd="worktrees/feature", check=True)
        
        # Verify feature workspace uses override
        result = run_workspace("config", "show", "feature")
        assert "feature-branch" in result.stdout
    
    def test_config_inheritance_chain(self, temp_workspace_git_enabled):
        """Test configuration priority: worktree > default > file."""
        # Test inheritance logic
        pass
```

**New test file: test_migration.py**
```python
class TestMigration:
    """Test migration from workspace.conf to git config."""
    
    def test_import_workspace_conf(self, temp_workspace_git_enabled):
        """Test importing existing workspace.conf."""
        # Create legacy workspace.conf
        config = Path("workspace.conf")
        config.write_text("http://example.com/repo.git main v1.0.0\n")
        
        # Import to workspace
        result = run_workspace("config", "import", "main")
        assert result.returncode == 0
        
        # Verify imported correctly
        config_check = subprocess.run(
            ["git", "config", "--worktree", "--get-all", "workspace.repo"],
            cwd="worktrees/main", capture_output=True, text=True
        )
        assert "http://example.com/repo.git main v1.0.0" in config_check.stdout
```

#### Test Execution Strategy

1. **Parallel Testing**: Existing test parallelization with pytest-xdist remains valid
2. **Fixture Reuse**: Class-scoped fixtures for expensive setup still applicable
3. **Backward Compatibility**: Tests verify both old (file) and new (git config) paths work

### Migration Strategy

#### Phase 1: Dual-Mode Support (Current + New)
- Update `workspace` script to support both configurations
- Existing workspaces continue using `workspace.conf`
- New workspaces can opt into git config

#### Phase 2: Automated Migration
```bash
# Migration command
workspace migrate [--all | workspace-name]
```
- Reads existing workspace.conf
- Imports to git config for specified/all workspaces
- Preserves workspace.conf for rollback

#### Phase 3: Deprecation
- Warning messages when workspace.conf is used
- Documentation updated to git config approach
- Grace period for users to migrate

#### Phase 4: Cleanup
- Remove workspace.conf support
- Simplify code to git-config only

## Benefits of This Approach

1. **Per-workspace flexibility**: Each workspace fully independent
2. **Version control**: Configuration changes tracked in git
3. **Atomic operations**: Git ensures consistency
4. **Standard tooling**: Uses git's native features
5. **Metadata support**: Rich workspace documentation
6. **Backward compatible**: Gradual migration path

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Nested worktrees complexity | Tested and confirmed working in git |
| Migration failures | Keep workspace.conf as fallback during transition |
| User confusion | Clear documentation and migration tools |
| Test suite breakage | Dual-mode fixtures support both configs |

## Implementation Timeline

1. **Branch Setup** ✅ (per-workspace-config branch created)
2. **Core Refactoring** (2-3 hours)
   - Initialize git repo
   - Update load_repos()
   - Modify switch_workspace()
3. **Config Commands** (1-2 hours)
   - Implement set/show/import
4. **Test Updates** (2-3 hours)
   - Update fixtures
   - Add new test cases
   - Verify backward compatibility
5. **Migration Tools** (1 hour)
   - Import command
   - Migration documentation
6. **Documentation** (1 hour)
   - Update README
   - Migration guide
   - Examples

## Success Criteria

- [ ] All existing tests pass with modifications
- [ ] New workspaces use git worktrees
- [ ] Per-workspace config working
- [ ] Migration from workspace.conf successful
- [ ] No breaking changes for existing users
- [ ] Documentation complete

## Next Session Context

This refactoring enables the key requirement: **independent repository configurations per workspace**, while maintaining backward compatibility and reusing most of the existing test infrastructure.