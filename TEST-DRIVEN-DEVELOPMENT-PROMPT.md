# Test-Driven Development Prompt for git-worktree-superproject

## Objective
Iteratively develop and fix tests and implementation until achieving:
- ✅ 100% test passing rate (233/233 tests)
- ✅ >90% code coverage on the workspace script
- ✅ All documented features validated by tests

## Current State
- **Repository**: `/home/tim/src/git-worktree-superproject`
- **Branch**: `per-workspace-config`
- **Test Status**: 632 passing, 66 failing, 1 skipped (90.5% pass rate)
- **Main Script**: `workspace` (bash script)
- **Test Framework**: pytest with fixtures in `test/conftest.py`

### Session Progress (2025-09-04)
- ✅ Fixed config invalid subcommand error handling (2 tests fixed)
- ✅ Fixed workspace directory resolution (WORKSPACE_ROOT vs SCRIPT_DIR)
- ✅ Fixed worktree test setup (added workspace.conf creation)
- ✅ Improved from 190/233 (82%) to 632/699 (90.5%)

## Iteration Strategy

### Phase 1: Fix Failing Tests (Priority)
Start each session with:
```bash
cd /home/tim/src/git-worktree-superproject
git checkout per-workspace-config
nix develop -c pytest --tb=short --lf  # Run last failed tests
```

#### Currently Failing Test Categories:

1. **Config Command Tests** (7 failures in `test_config_errors.py`)
   - Focus: Error handling, edge cases, Unicode support
   - Key Issues: Expected vs actual error messages, git initialization requirements
   
2. **Integration Workflows** (10 failures in `test_integration_workflows.py`)
   - Focus: Multi-repository coordination, CI/CD patterns
   - Key Issues: Worktree vs clone behavior, branch synchronization
   
3. **Per-Workspace Config** (11 failures in `test_per_workspace_config.py`)
   - Focus: Git worktree config, inheritance chain
   - Key Issues: Worktree-specific config not implemented

4. **Worktree Operations** (9 failures in `test_worktree_operations.py`)
   - Focus: Central repository management, worktree lifecycle
   - Key Issues: Implementation still uses clone instead of worktree

### Phase 2: Implementation Fixes
For each failing test:

1. **Understand the Failure**
   ```bash
   # Run specific failing test with verbose output
   nix develop -c pytest path/to/test.py::TestClass::test_method -vvs
   ```

2. **Examine Test Expectations**
   - Read the test to understand intended behavior
   - Check if test expectations match documented features
   - Determine if test or implementation needs fixing

3. **Fix Implementation or Test**
   ```bash
   # For implementation fixes
   $EDITOR workspace
   
   # For test fixes
   $EDITOR test/test_*.py
   
   # Verify fix
   nix develop -c pytest path/to/test.py::TestClass::test_method
   ```

4. **Verify No Regressions**
   ```bash
   # Run all tests in the modified test file
   nix develop -c pytest path/to/test.py
   
   # Run related test suites
   nix develop -c pytest test/test_workspace*.py
   ```

### Phase 3: Coverage Improvement

1. **Generate Coverage Report**
   ```bash
   nix develop -c pytest --cov=workspace --cov-report=term-missing
   ```

2. **Identify Uncovered Code**
   - Look for missing line numbers in coverage report
   - Focus on critical paths and error handling

3. **Write Targeted Tests**
   - Add tests to appropriate existing test files
   - Create new test files only if testing new features

### Iteration Loop Template

```markdown
## Iteration N: [Focus Area]

### 1. Current State
- Tests Passing: X/233
- Coverage: Y%
- Focus: [specific test file or feature]

### 2. Failing Test Analysis
Test: `test_name`
Expected: [what test expects]
Actual: [what actually happens]
Root Cause: [implementation bug | test expectation error | missing feature]

### 3. Fix Applied
[Describe the fix - either to test or implementation]

### 4. Verification
```bash
# Commands used to verify fix
```
Result: [PASS/FAIL]

### 5. Coverage Impact
Before: X%
After: Y%
New lines covered: [list any significant additions]

### 6. Next Target
[Which test to fix next and why]
```

## Test Categories to Focus On

### High Priority (Core Functionality)
1. **Workspace Operations** (`test_workspace.py`)
   - switch, sync, status, foreach, list, clean
   - Already mostly passing - ensure 100%

2. **Configuration Management** (`test_config.py`, `test_config_errors.py`)
   - config set/show/import/set-default
   - Inheritance and precedence rules
   - Error handling for invalid inputs

### Medium Priority (Advanced Features)
3. **Worktree Operations** (`test_worktree_operations.py`)
   - Central repository management
   - Worktree lifecycle (add/remove)
   - Performance vs clone operations

4. **Per-Workspace Config** (`test_per_workspace_config.py`)
   - Git worktree-specific configuration
   - Migration from legacy workspace.conf
   - Configuration isolation between workspaces

### Lower Priority (Complex Scenarios)
5. **Integration Workflows** (`test_integration_workflows.py`)
   - Multi-developer scenarios
   - CI/CD integration patterns
   - Large-scale repository management

6. **Edge Cases** (`test_workspace_advanced.py`, `test_superproject_edge_cases.py`)
   - Special characters, Unicode
   - Performance with many repositories
   - Network failure recovery

## Implementation Notes

### Key Areas Needing Implementation
1. **True Git Worktree Support**
   - Replace `git clone` with `git worktree add`
   - Implement central repository management
   - Update clean command to use `git worktree remove`

2. **Per-Workspace Git Config**
   - Implement worktree-specific config (`git config --worktree`)
   - Ensure proper config inheritance chain
   - Migration utilities from workspace.conf

3. **Error Handling**
   - Graceful handling of corrupted repositories
   - Network failure recovery
   - Disk space issues

## Success Criteria Checklist

- [ ] All tests passing (currently 632/699 = 90.5%)
- [ ] Code coverage >90% on workspace script
- [ ] No skipped tests without documentation (1 skipped)
- [x] All commands documented in README have tests
- [x] Performance tests show worktree advantages
- [ ] Migration path from clone-based to worktree-based tested
- [x] Error recovery scenarios validated
- [x] Multi-repository coordination verified

## Remaining Issues (66 failures)

### Categories Still Needing Work:
1. **Per-Workspace Config** (11 failures) - Git worktree config not fully implemented
2. **Config Edge Cases** (5 failures) - Unicode, special chars, complex inheritance
3. **Integration Workflows** (9 failures) - Complex multi-repo scenarios
4. **Some Worktree Tests** (8 failures) - Submodules, symlinks, large history
5. **Missing Coverage** (1 failure) - Repository naming conflicts

## Useful Commands

```bash
# Run all tests
nix develop -c pytest

# Run with parallel execution
nix develop -c pytest -n auto

# Run specific test file
nix develop -c pytest test/test_workspace.py

# Run tests matching pattern
nix develop -c pytest -k "config"

# Run with coverage
nix develop -c pytest --cov=workspace --cov-report=html

# Run only failing tests
nix develop -c pytest --lf

# Run tests that failed in last run first
nix develop -c pytest --ff

# Very verbose output for debugging
nix develop -c pytest -vvs test/test_file.py::test_name

# Show test durations
nix develop -c pytest --durations=10
```

## Session Start Checklist

1. [ ] Pull latest changes: `git pull`
2. [ ] Check current test status: `nix develop -c pytest --tb=no -q`
3. [ ] Review this prompt for current focus area
4. [ ] Run last failed tests: `nix develop -c pytest --lf`
5. [ ] Pick specific test to fix
6. [ ] Apply iterative development loop

## Progress Tracking

Update this section after each session:

### Session History
- **Session 1**: Fixed fixture issues, reduced failures from 61 to 42
- **Session 2 (2025-09-04)**: Major progress on worktree tests and config handling
  - Fixed config command error handling 
  - Fixed workspace directory resolution (WORKSPACE_ROOT vs SCRIPT_DIR)
  - Fixed worktree test setup by adding workspace.conf creation
  - Went from 190/233 tests (82%) to 632/699 tests (90.5%)

### Metrics Trend
| Date | Passing | Failing | Coverage | Notes |
|------|---------|---------|----------|-------|
| 2025-01-04 | 190 | 42 | TBD | Fixed fixtures |
| 2025-09-04 | 632 | 66 | TBD | Fixed worktree tests, config handling |

## End Goal

When all tests pass and coverage is satisfactory:
1. Run full test suite with coverage
2. Document any remaining gaps
3. Create maintenance guide for future test additions
4. Consider adding CI/CD integration for continuous testing