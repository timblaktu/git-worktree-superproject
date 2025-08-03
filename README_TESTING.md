# Testing wt-super

## Quick Start

Run all tests with a single command (no setup required):
```bash
uv run --extra test pytest
```

This command automatically:
- Creates a virtual environment  
- Installs pytest and test dependencies (pytest-cov, pytest-mock, pytest-timeout)
- Runs the comprehensive test suite (33 tests)
- Shows one line per test case with clear pass/fail status

## Alternative Setup

For manual virtual environment setup:
```bash
uv venv
source .venv/bin/activate
uv pip install -e ".[test]"  # Required - installs pytest and dependencies
pytest
```

## Common Commands

```bash
uv run --extra test pytest                         # Run all tests (one line per test)
uv run --extra test pytest -v                      # Verbose output with details
uv run --extra test pytest -q                      # Quiet output (summary only)
uv run --extra test pytest --cov=.                 # With coverage report
uv run --extra test pytest test/test_workspace.py  # Specific file
```

## Test Structure

- `test/conftest.py` - Pytest fixtures for test setup
  - `temp_workspace` - Creates isolated temporary directory
  - `git_repos` - Sets up mock git repositories
  - `workspace_config` - Generates test configuration
  - `run_workspace` - Helper to execute workspace commands
  
- `test/test_workspace.py` - Main test suite
  - Tests for all workspace commands (init, sync, status, exec, list, clean)
  - Edge cases and error handling
  
- `test/test_config.py` - Configuration parsing tests
  - Valid/invalid configuration formats
  - Comments and empty lines handling
  
- `test/fixtures/` - Test configuration files
  - `simple.conf` - Basic configuration
  - `complex.conf` - Advanced features (branches, pinned refs)
  - `invalid.conf` - Error cases

## Test Coverage

The test suite covers:
- All workspace commands and their options
- Configuration file parsing
- Error handling and edge cases
- Git repository operations
- Multi-workspace management
- Pinned dependencies
- Branch-specific configurations