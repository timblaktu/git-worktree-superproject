# Git Worktree Superproject - Unified Workspace Manager

## ⚠️ CRITICAL PROJECT-SPECIFIC RULES ⚠️
- **SESSION CONTINUITY**: Update this CLAUDE.md file with task progress and provide end-of-response summary of changes made
- **COMPLETION STANDARD**: Tasks complete ONLY when: (1) `git add` all files, (2) `cargo check` passes, (3) `cargo test` succeeds, (4) end-to-end functionality demonstrated. **Writing code ≠ Working system**
- **NEVER WORK ON MAIN OR MASTER BRANCH**: Current branch is `rust-migration` - continue development here
- **MANDATORY GIT COMMITS AT INFLECTION POINTS**: ALWAYS `git add` and `git commit` ALL relevant changes before finalizing your response to user
- **CONSERVATIVE TASK COMPLETION**: NEVER mark tasks as "completed" prematurely. Err on side of leaving tasks "in_progress" or "pending" for review in next session.
- **RUST-FIRST APPROACH**: This is now a Rust migration project - prioritize Rust solutions over bash/python patches
- **🚨 DESTRUCTIVE COMMAND SAFETY 🚨**:
  - NEVER use `rm -rf ~/` or `rm -rf /home/*` - these delete home directory
  - ALWAYS use FULL ABSOLUTE PATHS with destructive commands
  - ALWAYS test with `ls` before any `rm -rf`
  - A directory can be NAMED `~` (literal char) vs `~` (shell expansion to $HOME)
  - When in doubt: ask user before running ANY `rm -rf` command
- **🧪 TEST SAFETY RULES 🧪**:
  - ALL file-creating tests MUST use `tempdir()` for isolation
  - NEVER operate on project directories in tests
  - Path expansion tests (like test_expand_tilde) are string-only - NO file I/O
  - `cargo test` is SAFE - all tests use tempdir() or read-only operations
  - Before adding new filesystem tests: verify they use tempdir() or are read-only

---

## 🚀 **NEXT SESSION: Phase 3 - Python Test Migration (Continued)**

**Command to resume:** "Continue migrating Python tests - next file is test_per_workspace_config.py"

**What you'll do:**
1. Read and analyze `test/test_per_workspace_config.py` (11 tests)
2. Migrate git config system tests to Rust
3. Add tests to appropriate modules (likely config.rs or git.rs)
4. Continue building test coverage for production code

**Phase 2 Status:** ✅ **FULLY COMPLETE**
**Phase 3 Status:** 🔄 **IN PROGRESS** (Session 7: 12 tests migrated from test_config.py)
**Current State:** 18 tests passing (9 original + 9 new config tests), git clean

---

## 📊 **CURRENT SYSTEM STATUS**

**Current Branch**: `rust-migration` (created for unified Rust implementation)
**Build State**: 🔄 Rust migration in progress
**Architecture**: Multi-language system migrating to unified Rust:
- **Bash**: 1,481-line workspace management script (to be replaced)
- **Rust**: Production-ready AST-based Nix flake modification (`flake-input-modifier/`)
- **Python**: Comprehensive pytest test suite (728+ tests in `test/`)

**Migration Status**: ✅ Phase 1 COMPLETE | ✅ Phase 2 COMPLETE | 🔄 Phase 3 IN PROGRESS (12/728+ tests migrated)

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

#### **Phase 2: Advanced Features** (MOSTLY COMPLETE)
**Priority 1: Complete Worktree Operations** - ✅ COMPLETE
- ✅ Full worktree creation with validation and verification
- ✅ Safe worktree removal with uncommitted change detection
- ✅ Enhanced list command with branch and status info
- ✅ End-to-end lifecycle testing

**Priority 2: Nix Integration** - ✅ COMPLETE
- ✅ Nix flake URL modification via cmd_flake
- ✅ Integration with existing `flake-input-modifier` AST system
- ✅ Full workflow: read → AST modify → write
- ✅ End-to-end tested with complex flakes

**Priority 3: Configuration Management** - ✅ COMPLETE (Bug Fixed in Session 6)
- ✅ Git config integration (workspace.repo multi-value config)
- ✅ 3-tier inheritance chain (worktree → default → legacy)
- ✅ Full CLI commands (show, set, set-default, import)
- ✅ Worktree config extension support
- ✅ Import from workspace.conf files
- ✅ **CRITICAL FIX**: Worktree-specific config using ConfigLevel::Worktree

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

## 📋 **CURRENT TASKS** (2025-11-02 - Session 7: Phase 3 Test Migration Start)

**Status**: ✅ Phase 3 STARTED - First batch of workspace.conf parsing tests migrated!

✅ **Phase 2 ALL PRIORITIES COMPLETE**:
1. ✅ Tilde expansion in config paths (Priority 0 - Session 3)
2. ✅ Complete worktree operations (Priority 1 - Session 2)
3. ✅ Nix flake integration (Priority 2 - Session 3)
4. ✅ Configuration management system (Priority 3 - Session 5 + Session 6 bug fix)

**Completed This Session (Session 5 - Configuration Management)**:
- [x] ✅ Analyzed Python config test suite (60+ tests across 3 files)
- [x] ✅ Designed git config integration architecture (3-tier inheritance)
- [x] ✅ Implemented git config operations in GitOps (67 lines):
  - config_get_all(): Multi-value config reading
  - config_add(): Multi-value config writing
  - config_set(): Single-value config setting
  - config_unset_all(): Config removal
  - enable_worktree_config(): Extension enablement
  - is_worktree_config_enabled(): Extension checking
- [x] ✅ Added Config subcommand to CLI with 4 subcommands:
  - show: Display config with inheritance chain (60 lines)
  - set: Set worktree-specific config (49 lines)
  - set-default: Set superproject default config (27 lines)
  - import: Import from workspace.conf (58 lines)
- [x] ✅ Implemented 3-tier configuration inheritance:
  1. Worktree-specific (git config --worktree workspace.repo)
  2. Default (git config workspace.repo in superproject)
  3. Legacy workspace.conf file
- [x] ✅ All tests passing (cargo check ✅, cargo test ✅)
- [x] ✅ End-to-end testing:
  - workspace config show main ✅
  - workspace config set-default <url> <branch> ✅
  - Inheritance chain verified (default overrides legacy) ✅
- [x] ✅ Committed implementation (commit: 524a951)

**Code Changes Session 5**:
- workspace-manager/src/git.rs: +67 lines (git config integration)
- workspace-manager/src/cli.rs: +277 lines (config commands + CLI subcommand structure)
- Total: +344 lines of production code
- 1 commit: feature (524a951) - Phase 2 Priority 3 complete

**Completed This Session (Session 6 - Critical Bug Fix)**:
- [x] ✅ Researched libgit2 ConfigLevel API and found ConfigLevel::Worktree
- [x] ✅ Implemented 4 new worktree-specific config methods in GitOps:
  - worktree_config_add(): Add multi-value config at worktree level
  - worktree_config_set(): Set single-value config at worktree level
  - worktree_config_get_all(): Read from worktree level only
  - worktree_config_unset_all(): Remove worktree-level config
- [x] ✅ Updated CLI commands to use worktree-specific methods:
  - cmd_config_set(): Uses worktree_config_add()
  - cmd_config_show(): Uses worktree_config_get_all()
  - cmd_config_import(): Uses worktree-specific methods
- [x] ✅ All tests passing (cargo check ✅, cargo test ✅ - 9 tests)
- [x] ✅ End-to-end verification:
  - Config written to .git/worktrees/<name>/config.worktree ✅
  - Git CLI verification: git config --worktree --get-all workspace.repo ✅
  - Inheritance chain tested and working correctly ✅
  - Worktree config isolation verified ✅
- [x] ✅ Committed critical bug fix (commit: ecc914b)

**Code Changes Session 6**:
- workspace-manager/src/git.rs: +82 lines (worktree-specific config methods)
- workspace-manager/src/cli.rs: +7 lines (updated to use worktree methods)
- Total: +89 lines of production code
- 1 commit: bugfix (ecc914b) - CRITICAL FIX for worktree config

**Completed This Session (Session 7 - Phase 3 Test Migration Start)**:
- [x] ✅ Read and analyzed test/test_config.py (8 Python tests)
- [x] ✅ Added RepoConfig struct for workspace.conf parsing
- [x] ✅ Implemented parse_line() for individual config line parsing
- [x] ✅ Implemented parse_workspace_conf() for entire file parsing
- [x] ✅ Implemented to_config_value() for git config formatting
- [x] ✅ Created 12 comprehensive Rust tests:
  - test_simple_config: Parse URLs with default branches
  - test_config_with_branches: Parse URLs with specific branches
  - test_config_with_refs: Parse URLs with branch and ref (tag/commit)
  - test_config_comments_and_empty_lines: Ignore comments and whitespace
  - test_empty_configuration: Handle empty config files
  - test_only_newlines: Handle files with only newlines
  - test_only_comments: Handle files with only comments
  - test_only_whitespace: Handle files with only whitespace
  - test_comments_and_empty_lines_mixed: Mixed comment/empty line handling
  - test_config_missing_final_newline: Handle missing trailing newline
  - test_to_config_value: Format RepoConfig as git config value
  - test_parse_line_edge_cases: Edge cases for line parsing
- [x] ✅ All tests passing (cargo test: 18 passed, cargo check: clean)
- [x] ✅ Committed implementation (commit: d3c332a)

**Code Changes Session 7**:
- workspace-manager/src/config.rs: +53 lines (production: RepoConfig struct + parsing)
- workspace-manager/src/config.rs: +243 lines (tests: 12 test functions)
- Total: +296 lines (+53 production, +243 tests)
- 1 commit: feature (d3c332a) - Phase 3 Session 1 complete
- **Test Migration Progress**: 12/728+ tests migrated (1.6%)

**Previous Sessions Completed**:

**Session 4 - Safety Review**:
- [x] ✅ Critical safety concern raised by user about ~ directory
- [x] ✅ Analyzed all 6 tests - ALL SAFE (tempdir or read-only)
- [x] ✅ Verified NO tests operate on literal `~` directory
- [x] ✅ Added comprehensive TEST SAFETY RULES to CLAUDE.md

**Session 3**:
- [x] ✅ Fixed tilde expansion bug in config paths (workspace-manager/src/config.rs)
  - Imported FileSystem module into config
  - Applied expand_tilde() in default_config() (line 44)
  - Applied expand_tilde() in load() for TOML configs (lines 29-30)
  - Handles both worktree_base and main_repo paths
  - Verified with end-to-end test: worktree created in /home/tim/.worktrees/ ✅
  - Committed fix (commit: 6af5385)

- [x] ✅ Implemented Nix flake integration (workspace-manager/src/cli.rs)
  - Updated CLI args: added old_url parameter, changed new_url flag to -n
  - Implemented cmd_flake() using flake_input_modifier::replace_flake_input_url()
  - Full workflow: read flake.nix → AST modification → write back
  - Comprehensive error handling for file I/O and AST operations
  - End-to-end test 1: nixpkgs URL modified successfully
  - End-to-end test 2: home-manager URL with query params (?ref=) modified
  - Structure preservation verified (nested follows, formatting intact)
  - Committed implementation (commit: 9d0bf9d)

**Code Changes Session 3**:
- workspace-manager/src/config.rs: +9 lines (tilde expansion in configs)
- workspace-manager/src/cli.rs: +42 lines (flake integration, CLI args update)
- Total: +51 lines of production code
- 2 commits: bugfix (6af5385) + feature (9d0bf9d)

**Documentation Changes Session 4**:
- CLAUDE.md: +5 lines (TEST SAFETY RULES section)
- CLAUDE.md: +16 lines (Session 4 safety review documentation)
- Total: +21 lines of safety documentation
- 0 commits (documentation only, pending user review)

**Build Status**: ✅ cargo check passes | ✅ cargo test passes (18 tests: 9 original + 9 new) | ✅ All Phase 2 functionality working

**Phase 2 Priority Tasks**: ✅ **ALL COMPLETE**
1. [x] **PRIORITY 1**: Complete worktree operations - ✅ COMPLETE (Session 2)
2. [x] **PRIORITY 0**: Fix tilde expansion in config paths - ✅ COMPLETE (Session 3)
3. [x] **PRIORITY 2**: Integrate flake-input-modifier API into cmd_flake - ✅ COMPLETE (Session 3)
4. [x] **PRIORITY 3**: Configuration management with git config - ✅ COMPLETE (Session 5)
5. [x] **PRIORITY 0 (CRITICAL)**: Fix worktree-specific config storage bug - ✅ FIXED (Session 6)

**Phase 3 Test Migration**:
6. [x] **test_config.py (8 tests)**: ✅ COMPLETE (Session 7) - 12 Rust tests created
7. [ ] **test_per_workspace_config.py (11 tests)**: 🚀 NEXT - Git config system tests
8. [ ] **test_config_errors.py (21 tests)**: Pending - Error handling tests
9. [ ] **Remaining 688+ tests**: Pending - Worktree ops, integration, edge cases

⚠️ **CLEANUP TASK** (Safe to do manually):
- Leftover buggy directory: `/home/tim/src/git-worktree-superproject/~` (literal tilde name)
- Created by old bug before fix - contains empty `.worktrees` subdirectory
- **SAFE removal**: `rm -rf "/home/tim/src/git-worktree-superproject/~"` (from project root)
- **NEVER EVER**: `rm -rf ~/` (would delete entire home directory!)

## 🎯 **QUICK RESUME FOR NEXT SESSION**

**Resume Command**: `"Begin work on your top-priority task"`

**What Happens Next**:
You will read and migrate `test/test_per_workspace_config.py` (11 tests) to Rust, adding tests to the appropriate module (likely `workspace-manager/src/git.rs` since they test git config operations).

**Current State After Session 7**:
- ✅ **18 tests passing** (9 original + 9 new config parsing tests)
- ✅ **Clean build** (cargo check ✅, cargo test ✅)
- ✅ **Git clean** (all changes committed)
- ✅ **Phase 2 complete** (all priorities finished)
- 🔄 **Phase 3 started** (12/728+ tests migrated = 1.6%)

**Phase 3 Test Migration Strategy**:
1. ✅ **Config Tests - STARTED**:
   - ✅ test_config.py (8 tests → 12 Rust tests) - Session 7 COMPLETE
   - 🚀 test_per_workspace_config.py (11 tests) - **NEXT TARGET**
   - test_config_errors.py (21 tests) - Pending

2. **Worktree Operations Tests** (50+ tests):
   - test_worktree_operations.py (17 tests)
   - test_workspace.py (33 tests)
   - Already have working implementation to validate against

3. **Integration & Edge Cases** (53+ tests):
   - test_integration_workflows.py (10 tests)
   - test_workspace_advanced.py (17 tests)
   - test_superproject_edge_cases.py (12 tests)

**Test Migration Principles**:
- ✅ Use `tempdir()` for all file I/O tests (safety first)
- ✅ Follow existing test patterns from workspace-manager/src/*_tests.rs
- ✅ Port test logic, NOT bash script invocations
- ✅ Add tests to appropriate module files (config, git, fs)

**Architecture Decisions (Locked)**:
- Git Operations: libgit2-rs (not shell commands)
- Configuration: TOML with serde + workspace.conf parsing
- Error Handling: Custom WorkspaceError types
- Testing: Native Rust (migrating from Python)
- Nix Integration: Use existing flake-input-modifier library