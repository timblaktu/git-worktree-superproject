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

## 🚀 **NEXT SESSION: Phase 6 - Multi-Repo Workspace Feature Implementation (TDD)**

**Command to resume:** "Begin work on your top-priority task"

**What you'll do:**
1. Implement WorkspaceManager methods following classic TDD cycle (red → green → refactor)
2. Start with `switch()` - the foundation method most tests depend on
3. Watch tests turn green incrementally as features are implemented
4. Implement RealRepositoryOps using libgit2-rs for actual git operations
5. Track progress: X/28 tests passing as implementation progresses

**Phase 1-3 Status:** ✅ **COMPLETE** (Single-repo worktree functionality - 72 tests)
**Phase 4 Status:** ✅ **COMPLETE** (Multi-repo workspace test design - 28 test stubs)
**Phase 5 Status:** ✅ **COMPLETE** (Multi-repo workspace test implementation - full assertions)
**Current State:** 72 unit tests passing, 28 integration/property tests ready for TDD, cargo check ✅

---

## 📊 **CURRENT SYSTEM STATUS**

**Current Branch**: `rust-migration` (created for unified Rust implementation)
**Build State**: ✅ Single-repo worktree functionality complete
**Architecture**: Hybrid system - single-repo complete, multi-repo in design phase:
- **Bash**: 1,481-line workspace management script (multi-repo features - to be replaced)
- **Rust (Complete)**: Single-repo git worktree operations + Nix flake integration
- **Rust (Planned)**: Multi-repo workspace management (TDD approach)
- **Python**: Comprehensive pytest test suite (728+ tests - categorized by feature scope)

**Migration Status**:
- ✅ **Phase 1-3 COMPLETE**: Single-repo worktree operations (72 Rust tests passing)
- ✅ **Phase 4 COMPLETE**: Multi-repo workspace test design (28 test stubs, traits, fixtures)
- ✅ **Phase 5 COMPLETE**: Multi-repo workspace test implementation (full assertions, ready for TDD)
- 🚀 **Phase 6 NEXT**: Multi-repo workspace feature implementation (TDD - make tests pass)

## 🔧 **IMPORTANT PATHS**

1. **Workspace Script**: `/home/tim/src/git-worktree-superproject/workspace` (1,481 lines - migration target)
2. **Existing Rust AST**: `/home/tim/src/git-worktree-superproject/flake-input-modifier/` (to integrate)
3. **Python Test Suite**: `/home/tim/src/git-worktree-superproject/test/` (728+ tests to migrate)
4. **New Rust Manager**: `/home/tim/src/git-worktree-superproject/workspace-manager/` (Phase 1 complete)

## 🚧 **RUST MIGRATION STATUS** (2025-11-02 - Session 11: Phase Redefinition)

### **📋 MIGRATION PHASES REDEFINED** (Test-Driven Development Approach)

#### **✅ Phase 1-3: COMPLETE - Single-Repository Worktree Operations**

**Scope**: Git worktree operations for a SINGLE repository (not multi-repo workspaces)

**Phase 1: Core Infrastructure** - ✅ COMPLETE
- ✅ Git Operations Layer (libgit2-rs: worktrees, branches, status)
- ✅ Configuration System (TOML with serde, workspace.conf parsing)
- ✅ CLI Interface (clap with 8 subcommands: init, list, add, remove, info, branches, status, flake, config)
- ✅ File System Operations (directory management, tilde expansion, path resolution)

**Phase 2: Advanced Single-Repo Features** - ✅ COMPLETE
- ✅ Full worktree lifecycle (create, remove, list with validation)
- ✅ Nix flake integration (URL modification via flake-input-modifier AST)
- ✅ Configuration management (3-tier inheritance: worktree → default → legacy)
- ✅ Worktree-specific git config (ConfigLevel::Worktree)

**Phase 3: Single-Repo Test Coverage** - ✅ COMPLETE
- ✅ 67 Rust tests passing (5 original + 62 migrated)
- ✅ Config parsing tests (12 tests - workspace.conf validation)
- ✅ Git config system tests (11 tests - multi-value config, inheritance)
- ✅ Config error handling (22 tests - validation, edge cases, unicode)
- ✅ Worktree operations (17 tests - lifecycle, isolation, error handling)
- ✅ All tests use tempdir() for isolation
- ✅ Property-based testing patterns established

**Test Coverage Analysis**:
- Python tests migrated: 57 (all relevant to single-repo worktree operations)
- Rust tests created: 62 (with enhanced edge case coverage)
- Remaining 671+ Python tests: Test OLD bash script's multi-repo features (not yet implemented)

---

#### **🚀 Phase 4: NEXT - Multi-Repo Workspace Test Design** (Test-Driven Development)

**Scope**: Design Rust-idiomatic test architecture BEFORE implementing features

**Objective**: Create complete test suite for multi-repo workspace management using TDD approach

**Key Principle**: Following PYTEST_CARGO_MIGRATION.md guidance:
- ❌ Do NOT "lift and shift" Python/pytest patterns to Rust
- ✅ Design trait-based abstractions for testability
- ✅ Use Rust type system to eliminate type-safety tests
- ✅ Focus tests on BEHAVIOR and INVARIANTS, not implementation details
- ✅ Use rstest for fixtures, mockall for mocks, proptest for properties

**Deliverables** (1-2 sessions):
1. Define trait abstractions:
   - `RepositoryOps` trait (git operations abstraction for mocking)
   - `WorkspaceManager` trait (workspace lifecycle management)

2. Define domain types:
   - `Workspace` struct (collection of repos at specific branches)
   - `WorkspaceConfig` builder pattern (fluent configuration)
   - `RepoInfo`, `SyncReport`, `ForeachResult` (typed results)

3. Test infrastructure:
   - `TestWorkspace` fixture with Drop cleanup
   - Mock implementations using mockall
   - Property-based test strategies with proptest

4. Write ~20 test stubs (organized by category):
   - Core lifecycle: create, switch, remove workspace (5 tests)
   - Synchronization: pull repos, handle pinned repos (3 tests)
   - Bulk operations: foreach with closures (3 tests)
   - State validation: consistency invariants (4 tests)
   - Error handling: partial failures, rollback (5 tests)

5. Document test organization:
   - Unit tests: workspace-manager/src/workspace.rs (#[cfg(test)])
   - Integration tests: tests/multi_repo_workspace.rs
   - Property tests: invariant validation

**Reference**: All designs must integrate PYTEST_CARGO_MIGRATION.md principles

---

#### **✅ Phase 5: COMPLETE - Multi-Repo Workspace Test Implementation**

**Scope**: Implement the test suite designed in Phase 4 (tests will FAIL - no implementation exists)

**Deliverables** (1 session - Session 13):
1. ✅ Implemented mock-based unit tests (5 tests)
   - Used mockall to mock RepositoryOps trait
   - Tested business logic independent of git operations
   - Full expectations with .times() and .returning()

2. ✅ Implemented integration tests (19 tests)
   - Used real git operations in tempdir isolation
   - Tested full workspace lifecycle with real repos
   - Verified git state with actual git commands

3. ✅ Implemented property-based tests (4 properties)
   - Workspace consistency: atomic all-or-nothing operations
   - Sync idempotency: repeated syncs are no-ops
   - Foreach isolation: operations don't interfere
   - Branch name handling: all valid names work

4. ✅ Verified ALL tests compile and fail appropriately
   - Tests fail with "Phase 6: not yet implemented" errors
   - Validates test correctness before implementation
   - cargo check ✅, cargo test --lib ✅ (72 tests passing)

5. ✅ Documented test patterns
   - Complete assertions show expected behavior
   - Established coding standards for workspace tests
   - Ready for Phase 6 TDD implementation

**Success Criteria**: ✅ COMPLETE - Test suite exists, tests compile and fail cleanly, ready for TDD

---

#### **⏳ Phase 6: Multi-Repo Workspace Feature Implementation**

**Scope**: Implement multi-repo workspace features following TDD (make tests pass)

**Approach**: Classic TDD cycle:
1. Run tests → see failures
2. Implement minimal code to pass next test
3. Refactor for quality
4. Repeat until all tests green

**Deliverables** (4-6 sessions):
1. Implement domain types and traits
   - `Workspace`, `WorkspaceConfig`, `RepoInfo` structs
   - `RepositoryOps` and `WorkspaceManager` trait implementations

2. Implement WorkspaceManager operations
   - `switch()`: Create/switch to workspace with multiple repos
   - `sync()`: Pull updates from all repos
   - `foreach()`: Execute closures across repos
   - `list()`, `remove()`: Workspace management

3. Watch tests turn green incrementally
   - Track progress: X/20 tests passing
   - Refactor as patterns emerge

4. Add CLI commands
   - Delegate to WorkspaceManager (thin CLI layer)
   - Commands: workspace switch, sync, foreach, list, clean

5. End-to-end validation
   - Real-world usage scenarios
   - Performance verification
   - Documentation and examples

**Success Criteria**: All Phase 5 tests passing, CLI functional, ready for production use

### **🎯 SESSION 11 OBJECTIVES** (Phase Redefinition Complete)

**Primary Goal**: ✅ **COMPLETE** - Redefine migration phases with TDD approach
- ✅ Analyzed remaining Python tests (test_workspace.py, test_broken_repos.py)
- ✅ Discovered fundamental scope difference: single-repo vs multi-repo
- ✅ Marked Phases 1-3 COMPLETE for single-repo worktree operations
- ✅ Designed comprehensive Phase 4-6 strategy using PYTEST_CARGO_MIGRATION.md
- ✅ Established test-first approach: design tests → implement tests → implement features

**Secondary Goal**: ✅ **COMPLETE** - Document strategic insights
- ✅ Used sequential-thinking to design Rust-idiomatic test approach
- ✅ Defined trait abstractions for testability (RepositoryOps, WorkspaceManager)
- ✅ Reduced 33 Python tests → ~20 focused Rust tests via type safety
- ✅ Established property-based testing strategy for invariants

**Success Metrics**: ✅ ALL COMPLETE
- [x] Phase 1-3 marked COMPLETE (67 tests, single-repo scope)
- [x] Phase 4 fully designed (test architecture, trait abstractions)
- [x] Phase 5 fully planned (test implementation strategy)
- [x] Phase 6 fully planned (TDD feature implementation)
- [x] CLAUDE.md comprehensively updated with new direction
- [x] Ready to commit phase redefinition

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

## 📋 **CURRENT TASKS** (2025-11-02 - Session 13: Phase 5 Test Implementation Complete)

**Status**: ✅ **PHASE 5 COMPLETE** - All 28 multi-repo tests implemented with full assertions!

**Completed This Session (Session 13 - Phase 5: Multi-Repo Test Implementation)**:
- [x] ✅ Enhanced 5 unit tests with complete mockall assertions (workspace.rs)
  - test_sync_skips_pinned_repos: Verify pull() called only on non-pinned repos
  - test_sync_handles_pull_failures: Verify partial failure reporting
  - test_foreach_outside_workspace_errors: Verify error for nonexistent workspace
  - test_switch_with_empty_config: Verify empty config handling
  - test_remove_workspace: Verify successful workspace removal
- [x] ✅ Implemented 19 integration tests with real git verification (multi_repo_workspace.rs):
  - Category 1 (Core Lifecycle - 5 tests): Multi-repo creation, idempotency, pinned repos, branch names, removal
  - Category 2 (Synchronization - 3 tests): Pull updates, skip pinned repos, error handling
  - Category 3 (Bulk Operations - 3 tests): Foreach execution, environment variables, failure handling
  - Category 4 (State Inspection - 4 tests): Status reporting, workspace listing, modifications
  - Category 5 (Error Handling - 4 tests): Broken worktrees, uninitialized repos, detached HEAD, rollback
- [x] ✅ Implemented 4 property-based tests with invariant validation (workspace_properties.rs):
  - Property 1: Workspace consistency (atomic all-or-nothing operations)
  - Property 2: Sync idempotency (repeated syncs are no-ops without remote changes)
  - Property 3: Foreach isolation (commands don't interfere between repos)
  - Property 4: Branch name handling (all valid git branch names work correctly)
- [x] ✅ All tests compile successfully (cargo check ✅)
- [x] ✅ 72 unit tests passing (Phase 1-3 + Phase 5 unit tests)
- [x] ✅ Integration tests fail at expected points (setup calls switch() before Phase 6 implementation)
- [x] ✅ Committed complete test implementation (commit: e293ff7)

**Code Changes Session 13**:
- workspace-manager/src/workspace.rs: +67 lines (enhanced unit test assertions)
- workspace-manager/tests/multi_repo_workspace.rs: +393 lines net (+596 total with rewrites)
- workspace-manager/tests/workspace_properties.rs: +95 lines (property test invariants)
- Total: ~758 lines of comprehensive test logic
- 1 commit: feature (e293ff7) - Phase 5 COMPLETE

**Test Implementation Highlights**:
- **Mockall Pattern**: Full mock expectations with .times() and .returning() for business logic verification
- **Real Git Verification**: Integration tests verify actual git state with `git rev-parse`, `git describe`, etc.
- **Property-Based Strategy**: Generated inputs test invariants across many cases (10 cases per property)
- **Comprehensive Assertions**: All tests have complete assert! statements ready for Phase 6
- **Expected Behavior**: Integration tests panic at switch() in setup (correct - awaiting Phase 6)

**Build Status**: ✅ cargo check passes | ✅ cargo test --lib passes (72 tests) | ✅ Ready for Phase 6 TDD

---

## 📋 **PREVIOUS SESSION TASKS** (Session 10: Worktree Operation Tests Complete)

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
- **Test Migration Progress**: 8 Python tests → 12 Rust tests created

**Completed This Session (Session 8 - Git Config System Tests)**:
- [x] ✅ Read and analyzed test/test_per_workspace_config.py (11 Python tests)
- [x] ✅ Created helper function `create_test_repo()` for isolated test repositories
- [x] ✅ Migrated 11 comprehensive Rust tests to workspace-manager/src/git.rs:
  - test_worktree_specific_config: Worktree-specific git config validation
  - test_config_inheritance_chain: 3-tier inheritance (worktree > default > file)
  - test_workspace_isolation: Isolated worktree configurations
  - test_enable_worktree_config_extension: Extension enablement
  - test_config_add_and_get_all: Multi-value config operations
  - test_config_unset_all: Config removal operations
  - test_worktree_config_unset_all: Worktree-level config removal
  - test_worktree_config_set_single_value: Single-value worktree config
  - test_worktree_list_and_removal: Worktree lifecycle management
  - test_worktree_branch_creation: Branch creation with worktrees
  - (1 test covered by existing implementation)
- [x] ✅ All tests use tempdir() for safety (no project directory operations)
- [x] ✅ Proper worktree config extension enablement before worktree creation
- [x] ✅ All tests passing (cargo test: 28 passed, cargo check: ✅)
- [x] ✅ Committed implementation (commit: 292f15a)

**Code Changes Session 8**:
- workspace-manager/src/git.rs: +353 lines (tests only: 11 test functions + 1 helper)
- Total: +353 lines (100% test coverage improvements)
- 1 commit: feature (292f15a) - Phase 3 Session 2 complete
- **Test Migration Progress**: 11 Python tests → 11 Rust tests created

**Completed This Session (Session 9 - Config Error Handling Tests)**:
- [x] ✅ Read and analyzed test/test_config_errors.py (21 Python tests in 3 test classes)
- [x] ✅ Added validation functions to config.rs (28 lines production code):
  - validate_workspace_name(): Reject empty, path traversal (.., /, \)
  - validate_url(): Reject empty/whitespace-only URLs
  - Enhanced parse_line() with empty URL check
- [x] ✅ Created 22 comprehensive error handling tests (268 lines test code):
  - **Error Handling Tests (9 tests)**: Empty values, path traversal, file I/O errors
  - **Special Characters & Unicode (6 tests)**: Passwords, unicode, 1000+ char strings
  - **Format Edge Cases (7 tests)**: Tabs, git@/file:// URLs, relative paths
- [x] ✅ All tests passing (cargo test: 50 passed - up from 28)
- [x] ✅ Clean build (cargo check ✅)
- [x] ✅ Committed implementation (commit: a5d9e4a)

**Code Changes Session 9**:
- workspace-manager/src/config.rs: +28 lines (production: validation functions)
- workspace-manager/src/config.rs: +268 lines (tests: 22 test functions)
- Total: +296 lines (+28 production, +268 tests)
- 1 commit: feature (a5d9e4a) - Phase 3 Session 3 complete
- **Test Migration Progress**: 21 Python tests analyzed → 22 Rust unit tests created

**Completed This Session (Session 10 - Worktree Operation Tests)**:
- [x] ✅ Read and analyzed test/test_worktree_operations.py (17 Python tests in 4 test classes)
- [x] ✅ Created 17 comprehensive worktree operation tests (564 lines test code):
  - **TestWorktreeLifecycle (5 tests)**: Creation/removal, multiple worktrees, branch conflicts, orphaned cleanup, dirty worktrees
  - **TestCentralRepositoryManagement (4 tests)**: Repo creation, commit history, shared objects, references
  - **TestWorktreeErrorHandling (5 tests)**: Existing directory, nonexistent worktree, large changes, invalid branch names
  - **TestWorktreeIntegration (3 tests)**: Performance, multiple branches, isolation
- [x] ✅ All tests verify core worktree functionality:
  - .git file contains gitdir reference to central repo
  - Worktrees share git objects (commits visible from main repo)
  - Orphaned worktrees detected and pruned with validation
  - Status tracking with uncommitted changes
  - Error handling for invalid operations
- [x] ✅ All tests use tempdir() for safety (no project directory operations)
- [x] ✅ All tests passing (cargo test: 67 passed - up from 50)
- [x] ✅ Clean build (cargo check ✅ - warnings for unused code expected)
- [x] ✅ Committed implementation (commit: 093a583)

**Code Changes Session 10**:
- workspace-manager/src/git.rs: +564 lines (tests: 17 test functions)
- Total: +564 lines (100% test coverage - no production code changes needed)
- 1 commit: feature (093a583) - Phase 3 Session 4 complete
- **Test Migration Progress**: 17 Python tests → 17 Rust tests created

**Completed This Session (Session 12 - Phase 4: Multi-Repo Test Design COMPLETE)**:
- [x] ✅ Analyzed remaining Python tests (40 tests in test_workspace.py + test_broken_repos.py)
- [x] ✅ Designed trait-based architecture with mockall support:
  - `RepositoryOps` trait for git operation abstraction (#[cfg_attr(test, automock)])
  - `WorkspaceManager` trait for workspace lifecycle management
- [x] ✅ Defined domain types with builder pattern (1,127 lines total):
  - Core types: WorkspaceConfig, WorkspaceInfo, SwitchReport, SyncReport
  - Builder: WorkspaceConfigBuilder for fluent test setup
  - Status types: RepoStatus enum, StatusReport, WorkspaceStatus
  - Result types: ForeachResult, RepoCommandOutput
- [x] ✅ Created test infrastructure (tests/common/mod.rs, 213 lines):
  - TestWorkspace fixture with automatic Drop cleanup
  - TestGitRepos fixture for creating real bare git repositories
  - Helper functions for test repo management (create_repo, create_repo_with_tag, add_commit)
- [x] ✅ Wrote 28 test stubs organized by behavior:
  - **5 unit tests** (src/workspace.rs #[cfg(test)]): Using mockall mocks
  - **19 integration tests** (tests/multi_repo_workspace.rs): Using real git in tempdir
  - **4 property-based tests** (tests/workspace_properties.rs): Using proptest
- [x] ✅ All tests compile successfully (cargo check ✅)
- [x] ✅ Tests panic appropriately with "Phase 6: not yet implemented" messages
- [x] ✅ Created lib.rs to expose workspace-manager as library for tests
- [x] ✅ Updated Cargo.toml with test dependencies (mockall, rstest, proptest)

**Code Changes Session 12**:
- workspace-manager/src/workspace.rs: +447 lines (traits, types, builders, skeleton impl, 5 unit tests)
- workspace-manager/src/lib.rs: +11 lines (library interface)
- workspace-manager/tests/common/mod.rs: +213 lines (test infrastructure)
- workspace-manager/tests/multi_repo_workspace.rs: +531 lines (19 integration tests)
- workspace-manager/tests/workspace_properties.rs: +165 lines (4 property tests)
- workspace-manager/Cargo.toml: +4 lines (dev-dependencies + lib config)
- Total: +1,371 lines of test infrastructure
- **Test Count**: 28 new test stubs (5 unit + 19 integration + 4 property)

**Test Design Highlights** (Following PYTEST_CARGO_MIGRATION.md):
- Reduced 40 Python tests → 28 focused Rust tests via type safety
- Trait abstractions enable dependency injection and mocking
- Builder pattern simplifies complex test data setup
- Drop trait provides automatic cleanup (no explicit teardown)
- Property-based tests validate invariants across many inputs
- Tests focus on BEHAVIOR and INVARIANTS, not implementation details

**Phase 4 Test Organization**:
1. **Unit Tests** (workspace-manager/src/workspace.rs):
   - test_sync_skips_pinned_repos
   - test_sync_handles_pull_failures
   - test_foreach_outside_workspace_errors
   - test_switch_with_empty_config
   - test_remove_workspace

2. **Integration Tests** (tests/multi_repo_workspace.rs):
   - Core Lifecycle (5): create, idempotency, pinned repos, branches, removal
   - Synchronization (3): pull updates, skip pinned, errors
   - Bulk Operations (3): foreach execution, env vars, failures
   - State Inspection (4): status, list, empty state, modifications
   - Error Handling (4): broken worktrees, uninitialized, detached HEAD, partial failures

3. **Property Tests** (tests/workspace_properties.rs):
   - test_workspace_consistency_invariant (atomic operations)
   - test_sync_idempotency (sync twice = sync once)
   - test_foreach_isolation_invariant (no interference)
   - test_branch_name_handling (valid branch names)

**Cumulative Test Migration Progress**:
- Python tests analyzed: 57 single-repo + 40 multi-repo = 97 total
- Rust tests created: 62 single-repo + 28 multi-repo stubs = 90 total
- Percentage of 728+ Python tests: 13.3% (97/728)

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

**Build Status**: ✅ cargo check passes | ✅ cargo test passes (67 tests: 3 git + 2 fs original + 62 new = 67) | ✅ All Phase 2 functionality working

**Phase 2 Priority Tasks**: ✅ **ALL COMPLETE**
1. [x] **PRIORITY 1**: Complete worktree operations - ✅ COMPLETE (Session 2)
2. [x] **PRIORITY 0**: Fix tilde expansion in config paths - ✅ COMPLETE (Session 3)
3. [x] **PRIORITY 2**: Integrate flake-input-modifier API into cmd_flake - ✅ COMPLETE (Session 3)
4. [x] **PRIORITY 3**: Configuration management with git config - ✅ COMPLETE (Session 5)
5. [x] **PRIORITY 0 (CRITICAL)**: Fix worktree-specific config storage bug - ✅ FIXED (Session 6)

**Phase 3 Test Migration**:
6. [x] **test_config.py (8 tests)**: ✅ COMPLETE (Session 7) - 12 Rust tests created
7. [x] **test_per_workspace_config.py (11 tests)**: ✅ COMPLETE (Session 8) - 11 Rust tests created
8. [x] **test_config_errors.py (21 tests)**: ✅ COMPLETE (Session 9) - 22 Rust tests created
9. [x] **test_worktree_operations.py (17 tests)**: ✅ COMPLETE (Session 10) - 17 Rust tests created
10. [ ] **test_workspace.py (33 tests)**: 🚀 NEXT - Workspace integration tests
11. [ ] **Remaining 671+ tests**: Pending - Integration, edge cases, advanced features

⚠️ **CLEANUP TASK** (Safe to do manually):
- Leftover buggy directory: `/home/tim/src/git-worktree-superproject/~` (literal tilde name)
- Created by old bug before fix - contains empty `.worktrees` subdirectory
- **SAFE removal**: `rm -rf "/home/tim/src/git-worktree-superproject/~"` (from project root)
- **NEVER EVER**: `rm -rf ~/` (would delete entire home directory!)

## 🎯 **QUICK RESUME FOR NEXT SESSION**

**Resume Command**: `"Begin work on your top-priority task"`

**What Happens Next**: **Phase 6 - Multi-Repo Workspace Feature Implementation (TDD)**

Phase 5 test implementation is COMPLETE. Phase 6 is about implementing features to make tests pass:
1. All 28 tests are fully implemented with comprehensive assertions
2. Tests compile successfully but fail at unimplemented methods (as expected)
3. Phase 6 will implement features following classic TDD: red → green → refactor
4. Start with `switch()` - the foundation method that most tests depend on

**What you'll implement in Phase 6**:
- Implement `RealRepositoryOps` using libgit2-rs for actual git operations
- Implement `WorkspaceManagerImpl::switch()` to create/switch workspaces
- Implement `sync()`, `foreach()`, `list()`, `remove()`, `status()` incrementally
- Watch tests turn green as features are implemented
- Refactor for quality once tests pass

**Current State After Session 13** (Phase 5 Complete):
- ✅ **Phase 1-3 COMPLETE** (Single-repo worktree operations - 72 tests passing)
- ✅ **Phase 4 COMPLETE** (Multi-repo test design - 28 test stubs created)
- ✅ **Phase 5 COMPLETE** (Multi-repo test implementation - full assertions added)
- ✅ **All tests compile** (cargo check ✅, cargo test --lib ✅)
- ✅ **Unit tests passing** (72 tests: 67 Phase 1-3 + 5 Phase 5 multi-repo)
- ✅ **Integration tests ready** (19 tests with comprehensive assertions)
- ✅ **Property tests ready** (4 tests validating critical invariants)
- 🚀 **Ready for Phase 6** (TDD implementation)

**Phase Status Summary**:
1. ✅ **Phase 1-3 COMPLETE** - Single-repo worktree operations (72 tests passing)
2. ✅ **Phase 4 COMPLETE** - Multi-repo test design (28 test stubs, traits, types, fixtures)
3. ✅ **Phase 5 COMPLETE** - Multi-repo test implementation (full assertions, ready for TDD)
4. 🚀 **Phase 6 NEXT** - Multi-repo feature implementation (make tests pass via TDD)

**Key Files for Phase 6**:
- workspace-manager/src/workspace.rs (implement WorkspaceManager trait methods)
- workspace-manager/src/git.rs (may need additional git operations for multi-repo)
- Start with: `WorkspaceManagerImpl::switch()` - foundation for all other operations

**TDD Strategy for Phase 6**:
1. Run tests to see which fail first
2. Implement minimal code to make one test pass
3. Refactor for quality
4. Repeat until all 28 tests pass
5. Track progress: X/28 tests passing

**Expected First Task**: Implement `switch()` method to create workspaces with multiple repos, which will unblock most integration tests that call it in setup.