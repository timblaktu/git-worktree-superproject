#!/usr/bin/env bash
set -euo pipefail

# Unified Test Runner for git-worktree-superproject
# Runs both pytest (bash script tests) and cargo tests (Rust AST tests)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
PYTEST_EXIT=0
CARGO_EXIT=0

echo -e "${BLUE}🧪 Running Unified Test Suite for git-worktree-superproject${NC}"
echo "================================================================"

# Function to print section headers
print_section() {
    echo
    echo -e "${YELLOW}📋 $1${NC}"
    echo "----------------------------------------"
}

# Run pytest tests (bash script functionality)
print_section "Running pytest tests (bash script functionality)"
echo "Tests: ~728 tests covering workspace management, configuration, git operations"
echo

if command -v nix &> /dev/null && [[ -f flake.nix ]]; then
    echo "Using nix develop environment for pytest..."
    if nix develop --command pytest --tb=short -q; then
        echo -e "${GREEN}✅ Pytest tests passed${NC}"
    else
        PYTEST_EXIT=$?
        echo -e "${RED}❌ Pytest tests failed (exit code: $PYTEST_EXIT)${NC}"
    fi
else
    echo "No Nix environment detected, trying system pytest..."
    if command -v pytest &> /dev/null; then
        if pytest --tb=short -q; then
            echo -e "${GREEN}✅ Pytest tests passed${NC}"
        else
            PYTEST_EXIT=$?
            echo -e "${RED}❌ Pytest tests failed (exit code: $PYTEST_EXIT)${NC}"
        fi
    else
        echo -e "${RED}❌ pytest not found. Please install pytest or run in nix develop environment.${NC}"
        PYTEST_EXIT=1
    fi
fi

# Run cargo tests (Rust AST functionality)
print_section "Running cargo tests (Rust AST functionality)"
echo "Tests: AST-based flake input modification with structure preservation"
echo

cd flake-input-modifier
if cargo test; then
    echo -e "${GREEN}✅ Cargo tests passed${NC}"
else
    CARGO_EXIT=$?
    echo -e "${RED}❌ Cargo tests failed (exit code: $CARGO_EXIT)${NC}"
fi
cd ..

# Summary
echo
echo "================================================================"
print_section "Test Results Summary"

if [[ $PYTEST_EXIT -eq 0 ]]; then
    echo -e "${GREEN}✅ Pytest (bash script tests): PASSED${NC}"
else
    echo -e "${RED}❌ Pytest (bash script tests): FAILED (exit code: $PYTEST_EXIT)${NC}"
fi

if [[ $CARGO_EXIT -eq 0 ]]; then
    echo -e "${GREEN}✅ Cargo (Rust AST tests): PASSED${NC}"
else
    echo -e "${RED}❌ Cargo (Rust AST tests): FAILED (exit code: $CARGO_EXIT)${NC}"
fi

echo
if [[ $PYTEST_EXIT -eq 0 && $CARGO_EXIT -eq 0 ]]; then
    echo -e "${GREEN}🎉 All tests passed! The integration is working correctly.${NC}"
    exit 0
else
    echo -e "${RED}💥 Some tests failed. See details above.${NC}"
    exit 1
fi