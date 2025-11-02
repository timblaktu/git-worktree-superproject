# Git Worktree Superproject - Unified Workspace Manager

## ⚠️ CRITICAL PROJECT-SPECIFIC RULES ⚠️ 
- **SESSION CONTINUITY**: Update this CLAUDE.md file with task progress and provide end-of-response summary of changes made
- **COMPLETION STANDARD**: Tasks complete ONLY when: (1) `git add` all files, (2) `cargo check` passes, (3) `cargo test` succeeds, (4) end-to-end functionality demonstrated. **Writing code ≠ Working system**
- **NEVER WORK ON MAIN OR MASTER BRANCH**: Current branch is `rust-migration` - continue development here
- **MANDATORY GIT COMMITS AT INFLECTION POINTS**: ALWAYS `git add` and `git commit` ALL relevant changes before finalizing your response to user
- **CONSERVATIVE TASK COMPLETION**: NEVER mark tasks as "completed" prematurely. Err on side of leaving tasks "in_progress" or "pending" for review in next session.
- **RUST-FIRST APPROACH**: This is now a Rust migration project - prioritize Rust solutions over bash/python patches

## 📊 **CURRENT SYSTEM STATUS**

**Current Branch**: `rust-migration` (created for unified Rust implementation)
**Build State**: 🔄 Rust migration in progress
**Architecture**: Multi-language system migrating to unified Rust:
- **Bash**: 1,481-line workspace management script (to be replaced)
- **Rust**: Production-ready AST-based Nix flake modification (`flake-input-modifier/`)
- **Python**: Comprehensive pytest test suite (728+ tests in `test/`)

**Migration Status**: ✅ Phase 1 COMPLETE | 🚀 Phase 2 Priority 1 COMPLETE - Full worktree lifecycle implemented

## 🔧 **IMPORTANT PATHS**

1. **Workspace Script**: `/home/tim/src/git-worktree-superproject/workspace` (1,481 lines - migration target)
2. **Existing Rust AST**: `/home/tim/src/git-worktree-superproject/flake-input-modifier/` (to integrate)
3. **Python Test Suite**: `/home/tim/src/git-worktree-superproject/test/` (728+ tests to migrate)
4. **New Rust Manager**: `/home/tim/src/git-worktree-superproject/workspace-manager/` (Phase 1 complete)

## 🚧 **RUST MIGRATION STATUS** (2025-11-02)

### **📋 MIGRATION PHASES DEFINED**

#### **Phase 1: Core Infrastructure** (CURRENT PRIORITY)
**Components to implement**:
1. **Git Operations Layer**: libgit2-rs integration for repository management, worktree operations, branch management
2. **Configuration System**: TOML-based configuration with serde, environment detection, path resolution  
3. **CLI Interface**: clap-based argument parsing, command structure, help system
4. **File System Operations**: Directory management, file operations, permission handling

**Specific Phase 1 Actions**:
1. ✅ Project Setup: Repository prepared with rust-migration branch
2. ✅ Created Cargo workspace with workspace-manager and flake-input-modifier
3. ✅ Added all core dependencies (libgit2, clap, serde, tokio, etc.)
4. ✅ Defined module architecture: cli, config, error, git, fs
5. ✅ Implemented git operations with libgit2-rs (worktrees, branches, status)
6. ✅ Created comprehensive CLI with clap (8 subcommands)
7. ✅ All tests passing (9 tests total across workspace)
8. ✅ End-to-end CLI functionality verified

#### **Phase 2: Advanced Features** (IN PROGRESS)
**Priority 1: Complete Worktree Operations** - ✅ COMPLETE
- ✅ Full worktree creation with validation and verification
- ✅ Safe worktree removal with uncommitted change detection
- ✅ Enhanced list command with branch and status info
- ✅ End-to-end lifecycle testing

**Priority 2: Nix Integration** - NEXT
- Nix integration and flake operations
- Integration with existing `flake-input-modifier` AST system

**Priority 3: Configuration Management** - PENDING
- Repository management and state tracking
- Configuration detection and validation

#### **Phase 3: Testing and Polish** (FUTURE)  
- Migrate Python tests to native Rust testing
- Performance optimization and hardening
- Documentation and deployment

### **🎯 CURRENT SESSION OBJECTIVES**

**Primary Goal**: **Phase 1 Implementation Kickoff**
Begin implementing the core infrastructure for unified Rust workspace manager.

**Secondary Goal**: **Asset Integration Planning**
Plan integration of existing Rust AST system into new unified project.

**Success Metrics for Session (2025-11-02)**: ✅ ALL COMPLETE
- [x] New Rust project created with Cargo workspace structure
- [x] Core dependencies added (libgit2, clap, serde, tokio, etc.)
- [x] Basic module architecture defined (cli, config, error, git, fs)
- [x] Git operations foundation implemented with comprehensive API
- [x] CLI structure and parsing established (8 subcommands)
- [x] Integration completed for existing AST system (workspace dependencies)
- [x] All tests passing (cargo check ✅, cargo test ✅)
- [x] End-to-end functionality verified (workspace status, --help)

## 🔄 **IMPLEMENTATION STRATEGY**

### **Technical Decisions**
- **Project Structure**: Single Rust crate with modular architecture
- **Git Operations**: libgit2-rs for native performance (10-100x improvement potential)
- **Error Handling**: Custom error types with anyhow for development speed
- **Configuration**: TOML with serde for type-safe configuration
- **CLI**: clap for robust argument parsing and help generation
- **Testing**: Native Rust testing replacing multi-language coordination

### **Asset Integration Approach**
- **Preserve Existing**: Integrate current `flake-input-modifier` Rust AST system
- **Clean Slate**: New project structure without backwards compatibility overhead
- **Single-User Optimization**: No need for complex configuration compatibility
- **Test Migration**: Gradual migration of Python tests to native Rust tests

## 📋 **CURRENT TASKS** (2025-11-02 - Session 2 Complete)

**Status**: ✅ Phase 2 Priority 1 COMPLETE - Full worktree lifecycle implemented

⚠️ **CRITICAL ISSUE FOUND**: Tilde expansion in config paths - MUST FIX FIRST
⚠️ **FOR NEXT SESSION**: Fix tilde expansion, THEN begin Phase 2 Priority 2 - Nix flake integration

**Completed This Session (Session 2)**:
- [x] ✅ Enhanced cmd_add with comprehensive validation and verification (workspace-manager/src/cli.rs:259-341)
  - Pre-creation checks: worktree name exists, path collision detection
  - Branch existence validation with user feedback
  - Post-creation verification: validate worktree, verify directory exists
  - Clear success reporting with checkmarks
- [x] ✅ Implemented safe cmd_remove with uncommitted change detection (workspace-manager/src/cli.rs:343-406)
  - Safety checks: worktree exists, not locked, uncommitted changes
  - Force flag support for override
  - Filesystem directory removal
  - Fixed libgit2 WorktreeLockStatus handling
- [x] ✅ Enhanced cmd_list with branch and status info (workspace-manager/src/cli.rs:188-257)
  - Detailed mode: branch name, status counts, locked/valid state
  - Simple mode: branch names in brackets
  - Handles missing directories and detached HEAD
- [x] ✅ Added worktree_status() to GitOps (workspace-manager/src/git.rs:217-250)
  - Status summary for specific worktree paths
  - Used for safety checks in removal
- [x] ✅ End-to-end lifecycle testing
  - Created test worktree with new branch
  - Added files (uncommitted changes detected)
  - Removal blocked without force (safety check working)
  - Force removal successful (worktree and directory removed)
- [x] ✅ All changes committed (commit: 558a903)

**Code Changes**:
- +197 lines implementing robust worktree lifecycle (workspace-manager/src/cli.rs, git.rs)
- Fixed libgit2 WorktreeLockStatus enum handling
- Comprehensive error messages and user feedback

**Build Status**: ✅ cargo check passes | ✅ cargo test passes (9 tests)

**Phase 2 Priority Tasks**:
1. [x] **PRIORITY 1**: Complete worktree operations - ✅ COMPLETE
2. [ ] **PRIORITY 0**: Fix tilde expansion in config paths - ⚠️ BLOCKING ISSUE (10 min fix)
3. [ ] **PRIORITY 2**: Integrate flake-input-modifier API into cmd_flake - NEXT
4. [ ] **PRIORITY 3**: Implement configuration management (cmd_init)
5. [ ] **ONGOING**: Begin migrating Python tests to Rust

**Critical Issue Discovered**:
- **Problem**: Config uses literal `~/.worktrees` creating subdirectory instead of expanding to home
- **Evidence**: Test worktree created at `git-worktree-superproject/~/.worktrees/test-feature`
- **Expected**: Should be `/home/tim/.worktrees/test-feature`
- **Fix**: Apply `FileSystem::expand_tilde()` in Config::detect()
- **Priority**: HIGH - Must fix before continuing to Priority 2

## 🎯 **NEXT SESSION START**

**Quick Resume Command**: "Begin work on your top-priority task"

**Expected Action**:
**FIRST** - Fix tilde expansion bug (Priority 0 - BLOCKING):
- Read workspace-manager/src/config.rs
- Apply FileSystem::expand_tilde() to worktree_base in Config::detect()
- Test with config containing ~/
- Verify worktrees created in correct location
- Commit fix

**THEN** - Start Phase 2 Priority 2 - Nix Flake Integration:
- Read flake-input-modifier API documentation (flake-input-modifier/src/lib.rs:130-154)
- Implement cmd_flake using replace_flake_input_url() function
- Read flake.nix from filesystem, modify, write back
- Test with real flake.nix files
- Commit working implementation

**Current Focus**: Priority 0 (Bugfix) THEN Phase 2 Priority 2 - Nix Integration
**Critical Priority**: Integrate flake-input-modifier API into cmd_flake
**Strategic Goal**: Complete unified workspace manager with Nix flake support
**Innovation Opportunity**: Seamless worktree + flake modification workflow

**Architecture Decisions** (Locked):
- ✅ Git Operations: libgit2-rs (not shell commands)
- ✅ Configuration: TOML with serde
- ✅ Error Handling: Custom WorkspaceError types
- ✅ Testing: Native Rust (migrating from Python)
- ✅ Nix Integration: Use existing flake-input-modifier library

**Build Verification Before Starting**:
```bash
cargo check  # Should pass with 4 dead code warnings
cargo test   # Should pass all 9 tests
./target/release/workspace --version  # Should show v0.1.0
git status   # Should be clean on rust-migration branch
```