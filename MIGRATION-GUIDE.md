# Migration Guide: Moving to Per-Workspace Configuration

## Overview

The git-worktree-superproject tool now supports **per-workspace repository configurations**, allowing each workspace to have independent repository specifications tracked in git. This guide helps you migrate from the legacy single `workspace.conf` file to the new flexible configuration system.

## What's Changed

### Before (Single Configuration)
- One `workspace.conf` file shared by all workspaces
- All workspaces use identical repository specifications
- Configuration changes affect all workspaces

### After (Per-Workspace Configuration)
- Each workspace can have unique repository configurations
- Configurations stored in git config (version controlled)
- Three-tier configuration hierarchy:
  1. Workspace-specific (highest priority)
  2. Default (inherited by all)
  3. Legacy file (backward compatible)

## Migration Paths

### Option 1: Keep Using workspace.conf (No Action Required)

**Your existing setup continues to work!** The tool maintains full backward compatibility. If you're happy with a single configuration for all workspaces, no changes are needed.

### Option 2: Gradual Migration (Recommended)

Migrate workspace by workspace as needed:

1. **Import existing configuration to a workspace:**
```bash
./workspace config import main workspace.conf
```

2. **Verify the import:**
```bash
./workspace config show main
```

3. **Customize specific workspaces:**
```bash
# Main stays on stable
./workspace config set main https://github.com/org/app.git main

# Feature workspace uses feature branch
./workspace config set feature-x https://github.com/org/app.git feature-x
```

4. **Continue using workspace.conf as fallback for unmigrated workspaces**

### Option 3: Full Migration

Migrate all workspaces at once:

1. **Set default configuration for all workspaces:**
```bash
# Import all repos as defaults
while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ -z "${line// }" ]] && continue
    ./workspace config set-default $line
done < workspace.conf
```

2. **Import to specific workspaces that need overrides:**
```bash
./workspace config import feature-branch custom-feature.conf
```

3. **Rename workspace.conf to prevent confusion:**
```bash
mv workspace.conf workspace.conf.backup
```

## Migration Examples

### Example 1: Simple Project

**Before (workspace.conf):**
```
https://github.com/org/frontend.git
https://github.com/org/backend.git
https://github.com/org/database.git main v2.0
```

**After (git config):**
```bash
# Set as defaults for all workspaces
./workspace config set-default https://github.com/org/frontend.git main
./workspace config set-default https://github.com/org/backend.git main
./workspace config set-default https://github.com/org/database.git main v2.0
```

### Example 2: Feature Branch Development

**Scenario:** Feature branch needs different versions

```bash
# Main workspace (production stable)
./workspace config set main https://github.com/org/app.git main
./workspace config set main https://github.com/org/lib.git main v1.0.0

# Feature workspace (development versions)
./workspace config set feature https://github.com/org/app.git feature-new-api
./workspace config set feature https://github.com/org/lib.git main v2.0.0-beta
```

### Example 3: Team Collaboration

**Scenario:** Different team members need different subsets

```bash
# Frontend developer workspace
./workspace config set frontend-dev https://github.com/org/frontend.git develop
./workspace config set frontend-dev https://github.com/org/shared-ui.git develop
./workspace config set frontend-dev https://github.com/org/design-system.git main

# Backend developer workspace  
./workspace config set backend-dev https://github.com/org/backend.git develop
./workspace config set backend-dev https://github.com/org/database.git develop
./workspace config set backend-dev https://github.com/org/api-gateway.git main

# Full-stack workspace (everything)
./workspace config import fullstack workspace.conf
```

## Configuration Storage

The new system stores configurations in git config files:

- **Default (all workspaces):** `.git/config`
- **Workspace-specific:** `.git/worktrees/[workspace]/config.worktree`
- **Legacy fallback:** `workspace.conf`

View raw git config:
```bash
# View defaults
git config --get-all workspace.repo

# View workspace-specific (from within workspace)
cd worktrees/feature
git config --worktree --get-all workspace.repo
```

## Troubleshooting

### Issue: Workspace not picking up configuration

**Solution:** Check configuration priority:
```bash
./workspace config show [workspace-name]
```
This shows which configuration source is being used.

### Issue: Want to reset a workspace to defaults

**Solution:** Clear workspace-specific config:
```bash
cd worktrees/[workspace]
git config --worktree --unset-all workspace.repo
```

### Issue: Need to see all configurations

**Solution:** Use git config directly:
```bash
# Show all default configs
git config --get-regexp "^workspace\."

# Show workspace-specific configs
cd worktrees/[workspace]
git config --worktree --get-regexp "^workspace\."
```

## Benefits After Migration

1. **Independent Testing:** Test feature branches without affecting stable workspaces
2. **Version Control:** Configuration changes tracked in git history
3. **Team Flexibility:** Each developer can have custom workspace configurations
4. **Selective Updates:** Update specific workspaces without touching others
5. **Configuration Inheritance:** Set defaults once, override only where needed

## Best Practices

1. **Use defaults for common repositories:** Set repositories that all workspaces need as defaults
2. **Override only differences:** Workspaces inherit defaults, only set what's different
3. **Document workspace purposes:** Use meaningful workspace names (feature-api, release-v2, hotfix-security)
4. **Clean up old workspaces:** Remove unused workspaces to avoid confusion
5. **Version control workspace configs:** The configurations are now part of your git history

## Need Help?

- Run `./workspace config help` for command reference
- Check `./workspace help` for general usage
- Review the README.md for complete documentation
- File issues at: https://github.com/[your-org]/git-worktree-superproject

## Summary

The migration to per-workspace configuration is:
- **Optional** - Existing setups continue to work
- **Gradual** - Migrate workspace by workspace
- **Reversible** - Can always fall back to workspace.conf
- **Beneficial** - Enables more flexible development workflows

Start with one workspace, see the benefits, then migrate others as needed!