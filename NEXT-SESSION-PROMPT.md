# Prompt for Next Testing Session

## Use this prompt to start your next session with optimal context:

---

I need to fix failing tests and ensure comprehensive test coverage for the git-worktree-superproject tool. The test suite has been expanded to 233 tests but 61 are failing due to fixture issues.

**Repository:** `/home/tim/src/git-worktree-superproject`
**Branch:** `per-workspace-config`

## Current State
- Test suite expanded from ~150 to **233 total tests**
- **580+ test assertions passing**
- **61 tests failing** due to fixture issue in `temp_workspace`
- Three new comprehensive test files added:
  - `test_worktree_operations.py` (17 tests)
  - `test_config_errors.py` (21 tests) 
  - `test_integration_workflows.py` (10 tests)

## Immediate Priority: Fix Failing Tests

### Known Issue
The `temp_workspace` fixture in `conftest.py` has a directory creation conflict:
```python
FileExistsError: [Errno 17] File exists: '/tmp/.../test_workspace'
```

### Tests Currently Failing (61 total)
- All tests using `temp_workspace_git_enabled` fixture
- Primarily in new test files:
  - `test_worktree_operations.py` 
  - `test_config_errors.py`
  - `test_integration_workflows.py`

### Fix Required
Update `conftest.py` fixture to handle existing directories:
- Add `exist_ok=True` to `mkdir()` calls
- Or clean up existing directories before creation
- Ensure proper test isolation

## Second Priority: Validate All Commands Are Tested

### Core Commands Coverage Status
| Command | Current Coverage | Tests Location | Validation Needed |
|---------|-----------------|----------------|-------------------|
| **switch** | Good | Multiple files | Verify worktree creation |
| **sync** | Good | Multiple files | Test pinned repo skipping |
| **status** | Excellent | test_workspace.py | Confirm output format |
| **foreach** | Excellent | test_workspace.py | Test error propagation |
| **list** | Complete | test_workspace.py | ✅ Done |
| **clean** | Good | Multiple files | Test worktree removal |
| **help** | Basic | test_workspace.py | Verify all commands listed |

### Config Commands Coverage Status
| Command | Current Coverage | Tests Location | Validation Needed |
|---------|-----------------|----------------|-------------------|
| **config set** | Tested | test_config_errors.py | Test URL validation |
| **config show** | Tested | test_config_errors.py | Test inheritance |
| **config import** | Tested | test_config_errors.py | Test file parsing |
| **config set-default** | Tested | test_config_errors.py | Test global config |
| **config help** | Tested | test_config_errors.py | ✅ Done |

## Third Priority: Ensure Critical Workflows Pass

### Essential Workflows to Validate
1. **Basic Development Flow**
   - Create workspace → Make changes → Sync → Switch → Clean
   - Must work with both worktree and config systems

2. **Multi-Repository Coordination**
   - All repos switch branches together
   - Pinned repos remain fixed during sync
   - Status shows accurate state

3. **Configuration Precedence**
   - Workspace-specific > Default > Legacy file
   - Proper inheritance and override behavior

4. **Error Recovery**
   - Network failures don't corrupt state
   - Partial operations can be resumed
   - Clean removes worktrees properly

## Test Execution Strategy

### Step 1: Fix Fixtures
```bash
# Edit conftest.py to fix temp_workspace fixture
# Run tests to verify fixes
nix develop -c pytest -v --tb=short
```

### Step 2: Run Test Categories
```bash
# Core commands
nix develop -c pytest test/test_workspace*.py -v

# Config commands  
nix develop -c pytest test/test_config*.py -v

# Integration workflows
nix develop -c pytest test/test_integration_workflows.py -v

# Worktree operations
nix develop -c pytest test/test_worktree_operations.py -v
```

### Step 3: Coverage Analysis
```bash
# Generate coverage report
nix develop -c pytest --cov=workspace --cov-report=term-missing

# Identify untested code paths
# Add targeted tests for gaps
```

## Expected Outcomes

### Success Criteria
- ✅ All 233 tests passing
- ✅ No fixture-related failures
- ✅ Coverage >85% for workspace script
- ✅ All documented commands have tests
- ✅ Critical workflows validated

### Test Report Should Show
- **0 failed, 233+ passed**
- Execution time under 2 minutes
- Clean test output with no warnings
- Coverage report showing high percentages

## Files to Focus On

### Primary Files
- `test/conftest.py` - **Fix fixture issues here first**
- `workspace` - Main script being tested
- `test/test_worktree_operations.py` - New worktree tests
- `test/test_config_errors.py` - Config error handling
- `test/test_integration_workflows.py` - Complex workflows

### Supporting Files
- `TEST-COVERAGE-SUMMARY.md` - Current test achievements
- `README.md` - Command documentation
- `PER-WORKSPACE-CONFIG-DESIGN.md` - Architecture reference

## Debugging Tips

### For Fixture Issues
- Check if directories are being cleaned up properly
- Verify pytest's tmp_path handling
- Look for race conditions in parallel test execution
- Consider using pytest's built-in tmp_path fixture

### For Test Failures
- Use `-v` for verbose output
- Use `--tb=short` for concise tracebacks  
- Use `-k test_name` to run specific tests
- Use `--lf` to run last failed tests

## Final Validation

Once all tests pass, run the full suite with coverage:
```bash
# Full test suite with coverage
nix develop -c pytest --cov=workspace --cov-report=html --cov-report=term

# Parallel execution for speed
nix develop -c pytest -n auto

# Verify no regressions
git status  # Should show no unexpected changes
```

The goal is to achieve a fully passing test suite that comprehensively validates all workspace commands and workflows, providing confidence in the tool's reliability and correctness.

---

## Quick Start Commands
```bash
cd /home/tim/src/git-worktree-superproject
git checkout per-workspace-config

# Fix fixtures first
$EDITOR test/conftest.py

# Run all tests
nix develop -c pytest -v

# Run with coverage
nix develop -c pytest --cov=workspace --cov-report=term-missing
```