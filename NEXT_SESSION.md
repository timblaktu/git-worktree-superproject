# NEXT SESSION PROMPT - Phase 2 Implementation

## 📊 CURRENT STATE (2025-11-02)

**Branch**: `rust-migration`
**Last Commit**: `7d7bda3` - Update CLAUDE.md with Phase 1 completion status
**Build Status**: ✅ All systems green
- `cargo check` ✅ Passes (4 dead code warnings expected)
- `cargo test` ✅ 9 tests passing
- Binary: `target/release/workspace` ✅ Working (v0.1.0)

**Phase 1**: ✅ **COMPLETE** - Core infrastructure implemented and tested

## 🎯 TOP PRIORITY: Phase 2 Implementation

### Phase 2 Goal
Implement advanced features and actual worktree functionality using the foundation from Phase 1.

### Critical Understanding from Phase 1

**What Works Now:**
1. ✅ Cargo workspace structure (`workspace-manager` + `flake-input-modifier`)
2. ✅ Git operations API (libgit2-rs integration)
3. ✅ CLI framework (8 subcommands defined)
4. ✅ Configuration system (TOML with serde)
5. ✅ Error handling (custom types)
6. ✅ File system utilities

**What's Still Stubbed/Incomplete:**
- CLI commands are wired up but some have minimal implementation
- Nix flake integration (`cmd_flake`) just prints a message
- Configuration detection needs refinement
- No actual directory creation/cleanup in worktree operations
- Missing integration with `flake-input-modifier` library API

## 🔧 PHASE 2 TASKS (Priority Order)

### Priority 1: Complete Worktree Operations ⚡ IMMEDIATE
**Goal**: Make `workspace add` and `workspace remove` fully functional

**Tasks**:
1. [ ] Implement full worktree creation in `cmd_add`:
   - Create worktree directory structure
   - Handle branch creation/checkout properly
   - Verify worktree is valid after creation
   - Add error handling for edge cases

2. [ ] Implement full worktree removal in `cmd_remove`:
   - Check for uncommitted changes (unless --force)
   - Remove worktree directory from filesystem
   - Clean up git worktree references
   - Handle locked worktrees

3. [ ] Add `workspace list` enhancements:
   - Show current branch for each worktree
   - Display worktree status (clean/dirty)
   - Add sorting options

4. [ ] Test end-to-end:
   - Create test worktree
   - Verify filesystem structure
   - Make commits in worktree
   - Remove worktree cleanly

**Success Criteria**:
- Can create worktree: `workspace add feature-x --branch feature-x`
- Can list worktrees: `workspace list --detailed`
- Can remove worktree: `workspace remove feature-x`
- All operations reflected in both git and filesystem

### Priority 2: Nix Flake Integration 🔌 HIGH
**Goal**: Integrate existing `flake-input-modifier` library for flake operations

**Tasks**:
1. [ ] Implement `cmd_flake` using flake-input-modifier API:
   - Call `replace_flake_input_url()` from library
   - Read flake.nix from filesystem
   - Write modified content back
   - Add proper error handling

2. [ ] Add flake operations to worktree workflow:
   - Option to modify flake inputs when creating worktree
   - Auto-detect flake.nix in repository
   - Support multiple input modifications

3. [ ] Add new subcommand: `workspace flake-list`:
   - Parse and display current flake inputs
   - Show input URLs and configurations

**Success Criteria**:
- `workspace flake --input nixpkgs --url git+file:///path` works
- Flake modifications preserve formatting
- Integration with worktree creation optional but working

### Priority 3: Configuration Management 📝 MEDIUM
**Goal**: Make configuration system fully functional

**Tasks**:
1. [ ] Implement `cmd_init` fully:
   - Create config directory if missing
   - Write default configuration
   - Validate paths exist
   - Support interactive prompts

2. [ ] Enhance `Config::detect()`:
   - Check multiple config locations (.workspace.toml, ~/.config/workspace/)
   - Handle missing config gracefully with defaults
   - Support environment variables for overrides

3. [ ] Add configuration validation:
   - Verify worktree_base is writable
   - Check main_repo is valid git repository
   - Validate paths are absolute

**Success Criteria**:
- `workspace init` creates working configuration
- Commands work without explicit config (use defaults)
- Configuration errors provide helpful messages

### Priority 4: Testing & Quality 🧪 ONGOING
**Goal**: Begin migrating Python tests to Rust

**Tasks**:
1. [ ] Review Python test suite in `test/`:
   - Identify core test scenarios
   - Map to Rust test structure
   - Prioritize critical path tests

2. [ ] Add integration tests:
   - Test full worktree lifecycle
   - Test configuration management
   - Test error conditions

3. [ ] Add property-based tests:
   - Use proptest for edge cases
   - Test with various git states
   - Verify filesystem operations

**Success Criteria**:
- 20+ tests covering main workflows
- Integration tests for CLI commands
- Test coverage > 70% for core modules

## 📋 IMPLEMENTATION NOTES

### Key Files to Modify

**For Priority 1 (Worktree Operations)**:
- `workspace-manager/src/cli.rs` - cmd_add, cmd_remove, cmd_list
- `workspace-manager/src/git.rs` - May need additional git operations
- `workspace-manager/src/fs.rs` - Directory creation/removal helpers

**For Priority 2 (Nix Flake Integration)**:
- `workspace-manager/src/cli.rs` - cmd_flake implementation
- Integration with `flake-input-modifier` crate (already in dependencies)
- May need new module: `workspace-manager/src/nix.rs`

**For Priority 3 (Configuration)**:
- `workspace-manager/src/cli.rs` - cmd_init implementation
- `workspace-manager/src/config.rs` - Enhanced detection and validation

### Architecture Decisions Already Made

1. **Git Operations**: Use libgit2-rs (not shell git commands)
2. **Error Handling**: Custom WorkspaceError types (not anyhow everywhere)
3. **Configuration**: TOML format with serde
4. **Testing**: Native Rust tests (migrating from Python)
5. **Dependencies**: Workspace inheritance for consistency

### Current Module Structure
```
workspace-manager/src/
├── main.rs         - Entry point, logging init
├── cli.rs          - Command definitions and handlers
├── config.rs       - Configuration loading/saving
├── error.rs        - Error types
├── git.rs          - Git operations (libgit2-rs)
└── fs.rs           - File system utilities
```

## 🚨 CRITICAL REMINDERS

1. **Branch**: Stay on `rust-migration` - NEVER work on main
2. **Commits**: Commit at logical inflection points
3. **Testing**: Run `cargo check && cargo test` before commits
4. **Code Quality**: Address dead code warnings as features are used
5. **Integration**: Use existing `flake-input-modifier` API (don't rewrite)

## 🎯 SESSION START COMMAND

When resuming, respond to: **"Begin work on your top-priority task"**

**Expected Response**:
- Start with Priority 1, Task 1 (Complete worktree creation in cmd_add)
- Create todo list with TodoWrite
- Implement actual functionality
- Test with real repository
- Commit when working

## 📊 METRICS FOR PHASE 2 COMPLETION

Phase 2 will be complete when:
- [ ] Can create/remove worktrees with full filesystem operations
- [ ] Nix flake integration working end-to-end
- [ ] Configuration system fully functional
- [ ] 20+ tests passing (including integration tests)
- [ ] Documentation updated with examples
- [ ] All `cargo check` warnings addressed
- [ ] Ready for Phase 3 (Python test migration and polish)

## 🔗 KEY REFERENCES

- **Git Operations API**: `workspace-manager/src/git.rs:11-220`
- **CLI Framework**: `workspace-manager/src/cli.rs:1-300`
- **Flake Modifier API**: `flake-input-modifier/src/lib.rs:130-154` (replace_flake_input_url)
- **Original Bash Script**: `workspace` (1,481 lines - reference for behavior)
- **Test Suite**: `test/` directory (Python tests to migrate)

---

**Last Updated**: 2025-11-02 (Session end after Phase 1 completion)
