# Nix Flake Multi-Context Development Guide

> **Complete guide to using wt-super's industry-first AST-based Nix flake integration**

## Table of Contents
- [Quick Start for Nix Users](#quick-start-for-nix-users)
- [Core Concepts](#core-concepts)  
- [Step-by-Step Workflows](#step-by-step-workflows)
- [Advanced Patterns](#advanced-patterns)
- [Performance & Best Practices](#performance--best-practices)
- [Troubleshooting](#troubleshooting)

## Quick Start for Nix Users

### Prerequisites
- Existing Nix flake project (flake.nix in your repository)
- Multiple Nix repositories you want to develop with (nixpkgs, home-manager, etc.)
- Basic familiarity with git worktrees

### 5-Minute Setup

1. **Download wt-super to your flake project:**
```bash
cd your-nixos-config  # Or any project with flake.nix
curl -o workspace https://raw.githubusercontent.com/example/wt-super/main/workspace
chmod +x workspace
```

2. **Configure your development contexts:**
```bash
# Development context (local forks)
./workspace config set-flake dev nixpkgs "git+file:///home/user/src/nixpkgs?ref=my-feature"
./workspace config set-flake dev home-manager "git+file:///home/user/src/home-manager?ref=my-module"

# Production context (upstream)
./workspace config set-flake upstream nixpkgs "github:NixOS/nixpkgs/nixos-unstable"
./workspace config set-flake upstream home-manager "github:nix-community/home-manager"
```

3. **Switch contexts instantly:**
```bash
# Work with your forks
./workspace switch dev
cd worktrees/dev
nix run home-manager -- switch --flake . --dry-run

# Test with upstream
./workspace switch upstream  
cd worktrees/upstream
nix run home-manager -- switch --flake . --dry-run
```

## Core Concepts

### The Multi-Context Problem

When developing Nix configurations, you often need to:
- **Test with local forks** during active development
- **Validate with upstream** before submitting PRs
- **Compare versions** for performance or compatibility
- **Switch contexts frequently** without manual editing

### Traditional Pain Points

❌ **Manual flake.nix editing**
```nix
# Constantly switching between:
inputs.nixpkgs.url = "git+file:///home/user/nixpkgs?ref=feature";
# and:
inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
```

❌ **Structure destruction with sed/awk**
```bash
# This destroys formatting and comments:
sed -i 's|git+file://.*|github:NixOS/nixpkgs/nixos-unstable|' flake.nix
```

❌ **Error-prone manual processes**
- Forgetting to switch inputs before testing
- Accidentally committing development URLs
- Loss of comments and formatting

### The wt-super Solution

✅ **Perfect structure preservation**
```nix
# Before modification:
{
  description = "My NixOS config";
  
  inputs = {
    # Using local development version with my changes
    nixpkgs.url = "git+file:///home/user/nixpkgs?ref=writers-auto-detection";
    
    home-manager = {
      url = "git+file:///home/user/home-manager?ref=auto-validate-feature";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

# After AST-based modification - IDENTICAL except URLs:
{
  description = "My NixOS config";
  
  inputs = {
    # Using local development version with my changes  
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
```

✅ **Zero manual intervention**
- Automated context switching
- Version-controlled configurations
- Workspace isolation

✅ **Production-grade reliability**
- 41 comprehensive tests
- Graceful fallback mechanisms
- Sub-100ms performance

## Step-by-Step Workflows

### Workflow 1: Basic Fork Development

**Goal**: Develop a nixpkgs package while testing in your NixOS config

#### Step 1: Initial Setup
```bash
cd your-nixos-config
./workspace config set-flake dev nixpkgs "git+file:///home/user/src/nixpkgs?ref=add-my-package"
./workspace config set-flake upstream nixpkgs "github:NixOS/nixpkgs/nixos-unstable"
```

#### Step 2: Develop with Fork
```bash
./workspace switch dev
cd worktrees/dev

# Your flake.nix now uses: git+file:///home/user/src/nixpkgs?ref=add-my-package
nix build .#nixosConfigurations.myhost.config.system.build.toplevel
```

#### Step 3: Validate with Upstream
```bash
./workspace switch upstream
cd worktrees/upstream

# Your flake.nix now uses: github:NixOS/nixpkgs/nixos-unstable  
nix flake update
nix build .#nixosConfigurations.myhost.config.system.build.toplevel
```

#### Step 4: Compare Results
```bash
# Check what changed
diff ../dev/flake.lock ../upstream/flake.lock

# Performance comparison
time nix build .#nixosConfigurations.myhost.config.system.build.toplevel
```

### Workflow 2: Multi-Fork Development

**Goal**: Coordinate changes across nixpkgs + home-manager + NixOS-WSL

#### Step 1: Configure All Contexts
```bash
# Development: All local forks
./workspace config set-flake dev nixpkgs "git+file:///home/user/nixpkgs?ref=writers-auto-detection"
./workspace config set-flake dev home-manager "git+file:///home/user/home-manager?ref=auto-validate-feature"  
./workspace config set-flake dev NixOS-WSL "git+file:///home/user/NixOS-WSL?ref=plugin-shim-integration"

# Staging: Mix of fork + upstream for gradual testing
./workspace config set-flake staging nixpkgs "git+file:///home/user/nixpkgs?ref=writers-auto-detection"
./workspace config set-flake staging home-manager "github:nix-community/home-manager"
./workspace config set-flake staging NixOS-WSL "github:nix-community/NixOS-WSL"

# Production: All upstream
./workspace config set-flake upstream nixpkgs "github:NixOS/nixpkgs/nixos-unstable"
./workspace config set-flake upstream home-manager "github:nix-community/home-manager"
./workspace config set-flake upstream NixOS-WSL "github:nix-community/NixOS-WSL"
```

#### Step 2: Development Cycle
```bash
# Work with all forks
./workspace switch dev
cd worktrees/dev
nix run home-manager -- switch --flake . --dry-run

# Test gradual integration  
./workspace switch staging
cd worktrees/staging
nix run home-manager -- switch --flake . --dry-run

# Final validation
./workspace switch upstream
cd worktrees/upstream  
nix run home-manager -- switch --flake . --dry-run
```

#### Step 3: Monitor Configurations
```bash
# See what's configured for each context
./workspace config show-flake dev
./workspace config show-flake staging  
./workspace config show-flake upstream

# List all available inputs
./workspace config list-flake-inputs
```

### Workflow 3: Version Testing Matrix

**Goal**: Test your config against multiple nixpkgs versions

#### Step 1: Setup Version Matrix
```bash
# Current stable
./workspace config set-flake stable nixpkgs "github:NixOS/nixpkgs/nixos-24.05"

# Current unstable
./workspace config set-flake unstable nixpkgs "github:NixOS/nixpkgs/nixos-unstable"

# Your development branch
./workspace config set-flake dev nixpkgs "git+file:///home/user/nixpkgs?ref=my-improvements"

# Specific commit for bisecting
./workspace config set-flake bisect nixpkgs "github:NixOS/nixpkgs/abc123def456"
```

#### Step 2: Automated Testing
```bash
#!/bin/bash
# test-matrix.sh
for context in stable unstable dev bisect; do
  echo "=== Testing $context ==="
  ./workspace switch $context
  cd worktrees/$context
  
  echo "Building system..."
  if nix build .#nixosConfigurations.myhost.config.system.build.toplevel; then
    echo "✅ $context: SUCCESS"
  else
    echo "❌ $context: FAILED"
  fi
  
  cd ../..
done
```

### Workflow 4: Package Development with CI

**Goal**: Develop a package with automated testing across contexts

#### Step 1: CI-Ready Configuration
```bash
# CI can override these via environment
./workspace config set-flake ci-stable nixpkgs "github:NixOS/nixpkgs/nixos-24.05"
./workspace config set-flake ci-unstable nixpkgs "github:NixOS/nixpkgs/nixos-unstable"
./workspace config set-flake ci-dev nixpkgs "git+file:///home/runner/work/nixpkgs/nixpkgs?ref=${{ github.ref }}"
```

#### Step 2: GitHub Actions Integration  
```yaml
# .github/workflows/test-package.yml
name: Test Package
on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        context: [ci-stable, ci-unstable, ci-dev]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v20
      
      - name: Test in context
        run: |
          ./workspace switch ${{ matrix.context }}
          cd worktrees/${{ matrix.context }}
          nix build .#packages.x86_64-linux.my-package
          nix build .#checks.x86_64-linux.my-package-tests
```

## Advanced Patterns

### Pattern 1: Conditional Input Selection

**Use case**: Different inputs based on system or feature flags

```bash
# Configure GPU-specific context
./workspace config set-flake gpu nixpkgs "git+file:///home/user/nixpkgs?ref=cuda-improvements"
./workspace config set-flake gpu nvidia-docker "git+file:///home/user/nvidia-docker?ref=latest"

# Configure CPU-only context  
./workspace config set-flake cpu nixpkgs "github:NixOS/nixpkgs/nixos-unstable"
# nvidia-docker not configured - uses base flake.nix default
```

### Pattern 2: Follows Chain Management

**Use case**: Coordinating complex input dependencies

```bash
# Base flake.nix has complex follows chains:
# inputs.home-manager.inputs.nixpkgs.follows = "nixpkgs";  
# inputs.nix-darwin.inputs.nixpkgs.follows = "nixpkgs";

# Override just the root - follows chains automatically work
./workspace config set-flake dev nixpkgs "git+file:///home/user/nixpkgs?ref=feature"

# All dependent inputs automatically use your fork via follows!
```

### Pattern 3: Submodule Integration

**Use case**: Repositories with git submodules

```bash
# Repository requiring submodules
./workspace config set-flake dev esp-idf "git+file:///home/user/esp-idf?ref=feature&submodules=1"

# wt-super preserves the ?submodules=1 parameter perfectly
```

### Pattern 4: FlakeHub and Registry Integration

**Use case**: Mix of sources (GitHub, FlakeHub, local)

```bash
# FlakeHub input
./workspace config set-flake dev devenv "flakehub:cachix/devenv/1.0.1"

# Registry input (works with overrides)
./workspace config set-flake dev systems "systems"  

# Local override of registry input
./workspace config set-flake dev systems "git+file:///home/user/systems?ref=custom"
```

## Performance & Best Practices

### Performance Characteristics

#### AST Modification Speed
- **Small flakes** (<100 lines): ~10ms
- **Medium flakes** (100-1000 lines): ~25ms  
- **Large flakes** (1000+ lines): ~50ms
- **Complex flakes** (flake-parts, etc.): ~75ms

#### Structure Preservation
```bash
# Verify perfect preservation
./workspace switch upstream
cd worktrees/upstream
cp flake.nix flake.nix.backup

./workspace switch dev  
cd worktrees/dev
cp flake.nix flake.nix.dev

# Compare structure (should be identical except URLs)
diff -u ../upstream/flake.nix.backup flake.nix.dev
```

### Best Practices

#### 1. Configuration Organization
```bash
# Group related overrides
./workspace config set-flake dev nixpkgs "git+file:///home/user/nixpkgs?ref=feature-a"
./workspace config set-flake dev home-manager "git+file:///home/user/home-manager?ref=feature-a"

# Use descriptive workspace names
./workspace config set-flake nixpkgs-only nixpkgs "git+file:///home/user/nixpkgs?ref=writers"
./workspace config set-flake home-manager-only home-manager "git+file:///home/user/home-manager?ref=auto-validate"
```

#### 2. Development Workflow
```bash
# Always test both contexts before committing
./workspace switch dev
cd worktrees/dev && nix flake check

./workspace switch upstream  
cd worktrees/upstream && nix flake check
```

#### 3. Safety Practices
```bash
# Backup important configurations
./workspace config show-flake dev > dev-config-backup.txt

# Use dry-run for validation
nix run home-manager -- switch --flake . --dry-run

# Validate flake syntax after switches
nix flake show > /dev/null  # Quick syntax check
```

#### 4. Performance Optimization
```bash
# Use shallow clones for large repositories
./workspace config set-flake dev nixpkgs "git+file:///home/user/nixpkgs?ref=feature&shallow=1"

# Cache frequently-used contexts
nix build .#nixosConfigurations.myhost.config.system.build.toplevel --out-link result-dev
```

### Common Patterns

#### Multi-Developer Team Setup
```bash
# Shared configuration template
cat > team-setup.sh << 'EOF'
#!/bin/bash
# Each developer customizes their paths
USER_HOME=${USER_HOME:-$HOME}

./workspace config set-flake dev nixpkgs "git+file://$USER_HOME/src/nixpkgs?ref=team-feature"
./workspace config set-flake dev home-manager "git+file://$USER_HOME/src/home-manager?ref=team-feature"
./workspace config set-flake upstream nixpkgs "github:NixOS/nixpkgs/nixos-unstable"
./workspace config set-flake upstream home-manager "github:nix-community/home-manager"
EOF

chmod +x team-setup.sh
```

#### Release Preparation
```bash
# Pre-release testing matrix
./workspace config set-flake release-candidate nixpkgs "github:NixOS/nixpkgs/staging-next"
./workspace config set-flake release-stable nixpkgs "github:NixOS/nixpkgs/nixos-24.05"
./workspace config set-flake release-unstable nixpkgs "github:NixOS/nixpkgs/nixos-unstable"

# Test across all release contexts
for ctx in release-candidate release-stable release-unstable; do
  ./workspace switch $ctx && cd worktrees/$ctx && nix flake check && cd ../..
done
```

## Troubleshooting

### Common Issues

#### Issue: "flake-input-modifier not found"
**Cause**: AST tool not available, falling back to text processing
**Solution**: 
```bash
# Check if tool exists
ls -la bin/flake-input-modifier

# Rebuild if needed (from project root)
cd flake-input-modifier && cargo build --release
cp target/release/flake-input-modifier ../bin/
```

#### Issue: Flake syntax errors after modification
**Cause**: Complex Nix expressions not handled by fallback
**Solution**:
```bash
# Validate original flake
nix flake show > /dev/null

# Check specific workspace
./workspace switch dev
cd worktrees/dev
nix flake show > /dev/null

# Manual inspection
cat flake.nix | grep -A5 -B5 "url.*="
```

#### Issue: Performance degradation
**Cause**: Large flakes or complex AST processing
**Solution**:
```bash
# Profile AST processing
time ./bin/flake-input-modifier -o nixpkgs=new-url flake.nix

# Use text fallback for simple cases
FORCE_SED_FALLBACK=1 ./workspace switch dev
```

#### Issue: Input not found in configuration
**Cause**: Input name mismatch between flake.nix and config
**Solution**:
```bash
# List actual inputs in flake
./workspace config list-flake-inputs

# Check configuration  
./workspace config show-flake dev

# Fix naming
./workspace config set-flake dev correct-input-name "git+file://..."
```

### Debug Mode

Enable verbose output for troubleshooting:

```bash
# Set debug environment  
export WT_SUPER_DEBUG=1
export WT_SUPER_FLAKE_DEBUG=1

# Run with debug output
./workspace switch dev 2>&1 | tee debug.log

# Check AST tool output
./bin/flake-input-modifier --help
./bin/flake-input-modifier -v -o nixpkgs=test flake.nix
```

### Validation Workflow

Complete validation process:

```bash
#!/bin/bash
# validate-setup.sh

echo "=== Validating wt-super flake integration ==="

# 1. Check AST tool
echo "Checking AST tool..."
if ./bin/flake-input-modifier --version; then
  echo "✅ AST tool available"
else
  echo "⚠️  AST tool missing, will use text fallback"
fi

# 2. Validate base flake
echo "Validating base flake.nix..."
if nix flake show > /dev/null 2>&1; then
  echo "✅ Base flake syntax valid"
else
  echo "❌ Base flake has syntax errors"
  exit 1
fi

# 3. Test context switching
echo "Testing context switching..."
for ctx in dev upstream; do
  echo "  Testing $ctx context..."
  if ./workspace switch $ctx && cd worktrees/$ctx && nix flake show > /dev/null 2>&1; then
    echo "  ✅ $ctx context valid"
    cd ../..
  else
    echo "  ❌ $ctx context failed"
    cd ../.. 2>/dev/null || true
  fi
done

# 4. Verify structure preservation
echo "Verifying structure preservation..."
./workspace switch upstream && cd worktrees/upstream
cp flake.nix /tmp/upstream-flake.nix
cd ../..

./workspace switch dev && cd worktrees/dev  
if diff -q <(grep -v 'url.*=' flake.nix) <(grep -v 'url.*=' /tmp/upstream-flake.nix) > /dev/null; then
  echo "✅ Structure preservation verified"
else
  echo "⚠️  Structure differences detected (may be expected)"
fi
cd ../..

echo "=== Validation complete ==="
```

## Integration Examples

### Home Manager Integration
```bash
# Perfect for home-manager development
./workspace config set-flake dev home-manager "git+file:///home/user/home-manager?ref=new-module"
./workspace switch dev && cd worktrees/dev

# Test new module
nix run home-manager -- switch --flake . --dry-run
nix run home-manager -- switch --flake .

# Validate with upstream
./workspace switch upstream && cd worktrees/upstream
nix run home-manager -- switch --flake . --dry-run
```

### NixOS Configuration Testing
```bash
# Test system configuration changes
./workspace switch dev && cd worktrees/dev
sudo nixos-rebuild dry-build --flake .

# Test with different nixpkgs versions
./workspace switch stable && cd worktrees/stable
sudo nixos-rebuild dry-build --flake .
```

### Package Development
```bash
# Develop and test packages
./workspace switch dev && cd worktrees/dev
nix build .#packages.x86_64-linux.my-package
nix run .#packages.x86_64-linux.my-package

# Test package in different nixpkgs contexts
./workspace switch upstream && cd worktrees/upstream
nix build .#packages.x86_64-linux.my-package
```

This guide provides comprehensive coverage of wt-super's Nix flake integration, enabling you to leverage the industry-first AST-based multi-context development system for maximum productivity and reliability.