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

**Migration Status**: Phase 1 implementation ready to begin

## 🔧 **IMPORTANT PATHS**

1. **Workspace Script**: `/home/tim/src/git-worktree-superproject/workspace` (1,481 lines - migration target)
2. **Existing Rust AST**: `/home/tim/src/git-worktree-superproject/flake-input-modifier/` (to integrate)
3. **Python Test Suite**: `/home/tim/src/git-worktree-superproject/test/` (728+ tests to migrate)
4. **Target Location**: TBD - new Rust project structure within this repository

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
2. 🔧 **NEXT**: Create new Rust project structure
3. 🔧 **NEXT**: Add libgit2, clap, serde, tokio to Cargo.toml
4. 🔧 **NEXT**: Define main modules and interfaces
5. 🔧 **NEXT**: Implement basic repository and worktree operations
6. 🔧 **NEXT**: Create command parsing and help system

#### **Phase 2: Advanced Features** (FUTURE)
- Nix integration and flake operations
- Repository management and state tracking
- Integration with existing `flake-input-modifier` AST system

#### **Phase 3: Testing and Polish** (FUTURE)  
- Migrate Python tests to native Rust testing
- Performance optimization and hardening
- Documentation and deployment

### **🎯 CURRENT SESSION OBJECTIVES**

**Primary Goal**: **Phase 1 Implementation Kickoff**
Begin implementing the core infrastructure for unified Rust workspace manager.

**Secondary Goal**: **Asset Integration Planning**
Plan integration of existing Rust AST system into new unified project.

**Success Metrics for Current Session**:
- [ ] New Rust project created with proper structure
- [ ] Core dependencies added (libgit2, clap, serde)  
- [ ] Basic module architecture defined
- [ ] Git operations foundation implemented
- [ ] CLI structure and parsing established
- [ ] Integration plan for existing AST system

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

## 📋 **CURRENT TASKS** (2025-11-02)

**Status**: Ready to begin Phase 1 implementation in correct repository

**Immediate Tasks** (Next Session):
- [ ] Create new Rust project structure for unified workspace manager
- [ ] Add core dependencies (libgit2, clap, serde, tokio) to Cargo.toml  
- [ ] Define basic module architecture and interfaces
- [ ] Implement git operations foundation with libgit2-rs
- [ ] Create CLI structure and parsing with clap
- [ ] Plan integration of existing Rust AST system

**Implementation Focus**:
- **IMPLEMENTATION OVER PLANNING**: Write actual working code
- **ITERATIVE DEVELOPMENT**: Start simple, build incrementally  
- **TEST-DRIVEN APPROACH**: Write tests alongside implementation
- **PRESERVE EXISTING ASSETS**: Integrate current Rust AST code effectively

## 🎯 **SESSION CONTEXT**

**Current Focus**: Rust migration implementation Phase 1
**Critical Priority**: Begin core infrastructure implementation (git operations, configuration, CLI, filesystem)
**Strategic Goal**: Replace 1,481-line bash script with structured Rust implementation
**Innovation Opportunity**: Unified codebase with native performance improvements

**Architecture Decision Finalized**:
- ✅ **Approved**: Unified Rust implementation (libgit2, clap, serde, comprehensive testing)
- ❌ **Rejected**: Multi-language bash/rust/python coordination approach  
- 🔧 **Implementation**: Clean slate with existing Rust AST system integration

**Ready State**: All planning complete, repository prepared, ready for implementation