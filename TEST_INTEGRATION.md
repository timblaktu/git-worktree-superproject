# Test Integration Guide

## Overview

The git-worktree-superproject testing infrastructure consists of two complementary test suites:

1. **Pytest Suite** (bash script functionality) - 728+ tests
2. **Cargo Test Suite** (Rust AST functionality) - 3 tests  

This document describes how these test suites work together and how to run them.

## Test Architecture

### Component Separation

```
┌─────────────────────────────────────────────────────────────────┐
│                     git-worktree-superproject                   │
├─────────────────────────────────────────────────────────────────┤
│  Bash Script (workspace)          │  Rust Binary (flake-input- │
│  - Git worktree management         │   modifier)                │
│  - Configuration parsing           │  - AST-based flake parsing │
│  - Repository operations           │  - Surgical URL replacement│
│  - Multi-workspace coordination    │  - Structure preservation  │
├─────────────────────────────────────────────────────────────────┤
│  Pytest Tests                     │  Cargo Tests               │
│  - 728+ comprehensive tests       │  - 3 focused tests         │
│  - End-to-end workflows           │  - AST functionality       │
│  - Configuration validation       │  - CLI interface           │
│  - Error handling                 │  - URL replacement         │
└─────────────────────────────────────────────────────────────────┘
```

### Integration Points

The two components integrate at these interfaces:

1. **CLI Interface**: The bash script calls the Rust binary via command line
2. **File I/O**: Rust binary reads/writes flake.nix files that bash script manages
3. **Error Handling**: Both components provide compatible error codes and messages

## Test Suites

### Pytest Suite (Bash Script Tests)

**Location**: `test/` directory  
**Test Files**: 12 test files covering different aspects  
**Coverage**: ~728 tests (as of 2025-11-01)

**Key Test Areas**:
- **Workspace Management**: Creating, syncing, listing workspaces  
- **Configuration Parsing**: Valid/invalid configurations, comments handling
- **Git Operations**: Repository cloning, branch switching, worktree management
- **Error Scenarios**: Network failures, permission issues, corrupted repositories
- **Multi-workspace Workflows**: Complex dependency scenarios
- **Integration Testing**: End-to-end workflow validation

**Test Infrastructure**:
- **Fixtures**: Temporary workspace setup, mock git repositories
- **Isolation**: Each test runs in isolated temporary directories
- **Mocking**: Git repositories, network conditions, file system states

### Cargo Test Suite (Rust AST Tests)

**Location**: `flake-input-modifier/src/` files  
**Test Files**: Embedded in `lib.rs` and `main.rs`  
**Coverage**: 3 focused tests

**Test Areas**:
- **lib.rs**: 2 tests for core AST functionality
  - `test_simple_url_replacement`: Basic URL modification
  - `test_complex_url_replacement`: Advanced flake structures
- **main.rs**: 1 test for CLI interface
  - `test_cli_functionality`: Command-line argument processing

**Test Focus**:
- **AST Accuracy**: Verifies perfect structure preservation
- **URL Targeting**: Ensures only specified URLs are modified  
- **CLI Interface**: Validates command-line argument handling

## Running Tests

### Unified Test Runner

Use the provided script to run both test suites:

```bash
./run-tests.sh
```

This script:
1. Runs pytest tests in nix develop environment
2. Runs cargo tests in flake-input-modifier directory
3. Provides colored output and summary
4. Exits with proper error codes

### Individual Test Suites

**Pytest Only**:
```bash
nix develop --command pytest --tb=short -q
```

**Cargo Tests Only**:
```bash
cd flake-input-modifier && cargo test
```

### Test Coverage Analysis

**Pytest Coverage** (Completed 2025-11-01):
- ✅ **12 test files**: Comprehensive bash script testing
- ✅ **4,702 lines of test code**: Extensive coverage
- ✅ **728 passing tests**: End-to-end workflow validation
- ✅ **Sophisticated fixtures**: Isolated test environments

**Cargo Coverage** (Status):
- ✅ **3 basic tests**: Core AST functionality covered
- ⚠️  **Integration gap**: No testing of combined bash+Rust workflow
- ✅ **Fixed dependency**: tempfile dependency resolved (2025-11-01)

## Integration Testing Gaps

### Identified Gaps (2025-11-01)

1. **No Combined Workflow Testing**: 
   - Tests don't verify bash script → Rust binary communication
   - No validation of file handoff between components

2. **Missing Error Propagation Tests**:
   - How does bash script handle Rust binary failures?
   - Are error messages consistent between components?

3. **No Performance Integration Tests**:
   - Large flake files through full pipeline
   - Multi-input scenarios with complex AST structures

### Recommended Improvements

1. **Add Integration Test Suite**:
   ```bash
   test/test_integration_flake_modification.py
   ```
   - Test full workflow: workspace setup → flake modification → validation
   - Verify bash script properly calls Rust binary
   - Validate error handling across component boundaries

2. **Expand Rust Test Coverage**:
   - Add tests for complex flake structures (41 comprehensive tests already exist)
   - Test performance with large files
   - Validate edge cases in AST parsing

3. **Create Unified Coverage Reporting**:
   - Combine pytest coverage with cargo coverage
   - Identify gaps in integration scenarios

## Test Environment Setup

### Dependencies

**Pytest Requirements**:
- Python 3.12+ (provided by nix)
- pytest, pytest-cov, pytest-mock, pytest-timeout
- Git (for repository operations)

**Cargo Requirements**:
- Rust toolchain (provided by nix flake)
- tempfile dependency (fixed 2025-11-01)
- rnix, rowan crates for AST parsing

### Development Environment

The project includes a nix flake that provides all dependencies:

```bash
nix develop  # Enters environment with pytest, cargo, git, etc.
```

## Contributing

### Adding Tests

**For bash script functionality**:
1. Add test functions to appropriate `test/test_*.py` file
2. Use existing fixtures for workspace setup
3. Follow isolation patterns (temporary directories)

**For Rust AST functionality**:
1. Add test functions to `flake-input-modifier/src/lib.rs` or `main.rs`
2. Use existing patterns for AST manipulation tests
3. Include both positive and negative test cases

### Test Quality Standards

1. **Isolation**: Each test must be independent
2. **Deterministic**: Tests must produce consistent results
3. **Fast Execution**: Avoid unnecessary delays or complexity
4. **Clear Assertions**: Test intentions should be obvious
5. **Error Scenarios**: Include negative test cases

## Troubleshooting

### Common Issues

**"No such file or directory: './workspace'"**:
- This is expected in some broken repo tests
- Tests are designed to handle missing workspace script
- Should not affect overall test success

**"tempfile not found"**:
- Fixed in 2025-11-01 by adding tempfile dependency
- Run `cargo test` after dependency update

**Nix environment issues**:
- Use `nix develop` to enter proper environment
- Verify all dependencies are available

### Test Debugging

**Pytest debugging**:
```bash
nix develop --command pytest -v test/test_specific.py::test_function
```

**Cargo debugging**:
```bash
cd flake-input-modifier && cargo test -- --nocapture
```

## Future Improvements

1. **Enhanced Integration Testing**: Test combined bash+Rust workflows
2. **Performance Benchmarking**: Automated performance regression detection  
3. **Coverage Integration**: Unified coverage reporting across both languages
4. **CI/CD Integration**: Automated testing on multiple platforms
5. **Property-Based Testing**: Generate random flake structures for AST testing