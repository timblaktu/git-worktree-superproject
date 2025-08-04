# pytest-result-bar Plugin Migration Plan

## Overview

This document outlines the plan to migrate the pytest-result-bar plugin from the current monorepo into a standalone, shareable repository that can be distributed via PyPI and used across multiple projects.

## Current Status

The plugin currently exists as `test/pytest_result_bar.py` in the wt-super project and provides:
- Per-file progress bars with static positioning
- Unicode block characters for high-density test scenarios
- Support for both serial and parallel execution (pytest-xdist)
- Robust handling of different test execution phases
- Terminal width adaptation and cursor management

## Migration Goals

1. **Standalone Package**: Create an independent repository with proper Python packaging
2. **Easy Installation**: Enable `pip install pytest-result-bar` 
3. **Community Sharing**: Allow other projects to easily adopt the plugin
4. **Maintainability**: Separate development lifecycle and version management
5. **Enhanced Features**: Add configuration options and improved documentation

## Proposed Repository Structure

```
pytest-result-bar/
├── src/
│   └── pytest_result_bar/
│       ├── __init__.py           # Package entry point and version
│       ├── plugin.py             # Main plugin logic and pytest hooks
│       └── display.py            # DisplayManager class and terminal handling
├── tests/
│   ├── __init__.py
│   ├── test_plugin.py            # Core plugin functionality tests
│   ├── test_display.py           # Display manager tests
│   ├── test_integration.py       # End-to-end integration tests
│   └── fixtures/
│       ├── sample_tests.py       # Test files for integration testing
│       └── conftest.py           # Test fixtures and utilities
├── docs/
│   ├── README.md                 # Main documentation
│   ├── CHANGELOG.md              # Version history
│   ├── CONFIGURATION.md          # Configuration options
│   └── examples/
│       ├── basic_usage.py
│       ├── custom_config.py
│       └── ci_integration.md
├── pyproject.toml                # Modern Python packaging configuration
├── LICENSE                       # MIT License
├── .gitignore
├── .github/
│   └── workflows/
│       ├── test.yml              # CI testing pipeline
│       ├── publish.yml           # PyPI publishing
│       └── lint.yml              # Code quality checks
└── scripts/
    └── release.py                # Release automation
```

## Package Configuration (pyproject.toml)

```toml
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "pytest-result-bar"
dynamic = ["version"]
description = "A pytest plugin that shows per-file progress bars with xdist support"
readme = "docs/README.md"
license = "MIT"
authors = [
    {name = "Your Name", email = "your.email@example.com"},
]
classifiers = [
    "Development Status :: 4 - Beta",
    "Framework :: Pytest",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Operating System :: OS Independent",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.8",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Topic :: Software Development :: Testing",
]
keywords = ["pytest", "progress", "testing", "xdist", "parallel"]
dependencies = [
    "pytest>=6.0.0",
]
requires-python = ">=3.8"

[project.optional-dependencies]
dev = [
    "pytest-xdist",
    "pytest-cov",
    "black",
    "ruff",
    "mypy",
    "hatch",
]

[project.urls]
Homepage = "https://github.com/yourusername/pytest-result-bar"
Repository = "https://github.com/yourusername/pytest-result-bar"
Documentation = "https://github.com/yourusername/pytest-result-bar#readme"
"Bug Tracker" = "https://github.com/yourusername/pytest-result-bar/issues"

[project.entry-points.pytest11]
result_bar = "pytest_result_bar.plugin"

[tool.hatch.version]
path = "src/pytest_result_bar/__init__.py"

[tool.ruff]
line-length = 100
target-version = "py38"

[tool.ruff.lint]
select = ["E", "F", "W", "I", "N", "UP", "YTT", "S", "BLE", "FBT", "B", "A", "COM", "C4", "DTZ", "T10", "EM", "EXE", "FA", "ISC", "ICN", "G", "INP", "PIE", "T20", "PYI", "PT", "Q", "RSE", "RET", "SLF", "SLOT", "SIM", "TID", "TCH", "INT", "ARG", "PTH", "TD", "FIX", "ERA", "PD", "PGH", "PL", "TRY", "FLY", "NPY", "AIR", "PERF", "FURB", "LOG", "RUF"]

[tool.mypy]
python_version = "3.8"
strict = true
warn_return_any = true
warn_unused_configs = true
```

## Code Organization Strategy

### Module Separation
1. **`src/pytest_result_bar/plugin.py`**:
   - `ResultBarReporter` class
   - All pytest hook functions
   - Plugin configuration and initialization

2. **`src/pytest_result_bar/display.py`**:
   - `DisplayManager` class
   - Terminal display logic
   - Unicode character handling
   - Cursor positioning and layout management

3. **`src/pytest_result_bar/__init__.py`**:
   - Package version and metadata
   - Public API exports
   - Entry point configuration

### Key Improvements for Standalone Package

#### Enhanced Configuration Support
```python
# Support multiple configuration methods:
# 1. Command line: --result-bar --result-bar-width=80
# 2. pytest.ini: [tool:pytest] addopts = --result-bar
# 3. pyproject.toml: [tool.pytest.ini_options] result_bar = true
# 4. Environment: PYTEST_RESULT_BAR=1
```

#### Better Error Handling
- Graceful degradation for unsupported terminals
- Enhanced Unicode/encoding issue handling  
- Robust fallbacks for terminal capability detection
- Better error messages for common issues

#### Advanced Features
- Configurable progress bar characters
- Custom color schemes
- Optional file grouping/filtering
- Integration with popular CI systems
- Performance metrics and timing information

## Testing Strategy

### Test Coverage Areas
1. **Core Plugin Functionality**:
   - Test collection and counting
   - Result processing across all phases (setup, call, teardown)
   - Parallel execution handling (pytest-xdist integration)

2. **Display Management**:
   - Unicode character progression
   - Terminal width adaptation
   - Cursor positioning and layout calculation
   - Color prioritization logic

3. **Integration Testing**:
   - Real test execution scenarios
   - Different test file structures
   - Error conditions and edge cases
   - Performance under load

4. **Compatibility Testing**:
   - Multiple Python versions (3.8-3.12)
   - Different pytest versions (6.0+)
   - Various terminal environments
   - Operating systems (Linux, macOS, Windows)

### Test Infrastructure
```python
# tests/conftest.py - Shared fixtures
@pytest.fixture
def mock_terminal():
    """Mock terminal with configurable width."""
    
@pytest.fixture
def sample_test_files():
    """Generate test files with known outcomes."""

# tests/test_integration.py - End-to-end tests  
def test_serial_execution_progress():
    """Test progress bars in serial mode."""
    
def test_parallel_execution_progress():
    """Test progress bars with pytest-xdist."""
    
def test_unicode_rendering():
    """Test Unicode block character progression."""
```

## CI/CD Pipeline

### GitHub Actions Workflows

#### Testing Pipeline (.github/workflows/test.yml)
```yaml
name: Test
on: [push, pull_request]
jobs:
  test:
    strategy:
      matrix:
        python-version: ['3.8', '3.9', '3.10', '3.11', '3.12']
        os: [ubuntu-latest, macos-latest, windows-latest]
        pytest-version: ['6.0', '7.0', '8.0', 'latest']
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}
      - run: pip install -e .[dev]
      - run: pytest tests/ --cov=pytest_result_bar
      - run: pytest --result-bar tests/  # Self-test
```

#### Publishing Pipeline (.github/workflows/publish.yml)
```yaml
name: Publish
on:
  release:
    types: [published]
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
      - run: pip install hatch
      - run: hatch build
      - run: hatch publish
        env:
          HATCH_INDEX_USER: __token__
          HATCH_INDEX_AUTH: ${{ secrets.PYPI_API_TOKEN }}
```

## Migration Implementation Steps

### Phase 1: Repository Setup
1. Create new GitHub repository `pytest-result-bar`
2. Set up basic directory structure
3. Initialize git repository with proper `.gitignore`
4. Create initial `pyproject.toml` configuration

### Phase 2: Code Extraction and Refactoring
1. **Extract `DisplayManager`** from current plugin:
   - Move to `src/pytest_result_bar/display.py`
   - Clean up imports and dependencies
   - Add proper docstrings and type hints

2. **Extract `ResultBarReporter`** and hooks:
   - Move to `src/pytest_result_bar/plugin.py`  
   - Refactor hook functions for clarity
   - Add configuration handling

3. **Create package entry point**:
   - Set up `src/pytest_result_bar/__init__.py`
   - Define public API and version management

### Phase 3: Enhanced Features
1. **Configuration System**:
   - Add support for `pytest.ini`, `pyproject.toml`, environment variables
   - Command-line options for customization
   - Runtime configuration validation

2. **Improved Error Handling**:
   - Better terminal capability detection
   - Graceful fallbacks for encoding issues
   - Enhanced debugging output

3. **Documentation**:
   - Comprehensive README with examples
   - API documentation for customization
   - Integration guides for CI systems

### Phase 4: Testing and Quality
1. **Comprehensive Test Suite**:
   - Unit tests for all components
   - Integration tests with real pytest scenarios
   - Performance and stress testing

2. **Code Quality**:
   - Set up linting (ruff, black)
   - Type checking (mypy)
   - Coverage reporting

3. **CI/CD Pipeline**:
   - Automated testing across environments
   - Release automation
   - Documentation generation

### Phase 5: Release and Distribution
1. **PyPI Publishing**:
   - Package building and validation
   - Initial release (v1.0.0)
   - Documentation on PyPI

2. **Community Setup**:
   - Issue templates
   - Contributing guidelines
   - Code of conduct

## Usage After Migration

### Installation
```bash
pip install pytest-result-bar
```

### Basic Usage
```bash
# Enable result bars
pytest --result-bar

# With parallel execution
pytest --result-bar -n auto

# With custom configuration
pytest --result-bar --result-bar-width=120
```

### Configuration Example (pyproject.toml)
```toml
[tool.pytest.ini_options]
addopts = "--result-bar"

[tool.pytest_result_bar]
width = 100
show_count = true
unicode_blocks = true
```

## Benefits of Migration

1. **Easier Adoption**: Simple `pip install` for any project
2. **Version Management**: Semantic versioning and release notes
3. **Community Growth**: Issues, PRs, and feature requests from users
4. **Maintenance**: Independent development and testing lifecycle
5. **Distribution**: Available on PyPI for worldwide access
6. **Integration**: Easy to add to CI/CD pipelines and Docker images

## Timeline Estimation

- **Phase 1** (Repository Setup): 1-2 days
- **Phase 2** (Code Extraction): 3-5 days  
- **Phase 3** (Enhanced Features): 5-7 days
- **Phase 4** (Testing/Quality): 7-10 days
- **Phase 5** (Release): 2-3 days

**Total Estimated Time**: 3-4 weeks for complete migration

## Future Enhancements

Post-migration features to consider:
- Integration with popular test runners beyond pytest
- Web dashboard for CI environments
- Metrics collection and reporting
- Custom themes and color schemes
- Plugin ecosystem for extensions
- Performance profiling integration

---

This migration plan provides a roadmap for transforming the current pytest-result-bar plugin into a professional, standalone package that can benefit the broader Python testing community.