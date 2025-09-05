#!/usr/bin/env bash
# Test script for workspace completions

echo "Testing workspace completion setup..."
echo

# Test bash completion
echo "=== Testing Bash Completion ==="
if bash -c "source ./workspace-completion.bash 2>/dev/null && complete -p workspace" &>/dev/null; then
    echo "✓ Bash completion loads successfully"
    
    # Test that completion function exists
    if bash -c "source ./workspace-completion.bash && type _workspace_completions" &>/dev/null; then
        echo "✓ Completion function '_workspace_completions' is defined"
    else
        echo "✗ Completion function not found"
    fi
else
    echo "✗ Failed to load bash completion"
fi
echo

# Test zsh completion  
echo "=== Testing Zsh Completion ==="
if zsh -c "source ./workspace-completion.zsh 2>/dev/null && functions | grep -q '^_workspace '" &>/dev/null; then
    echo "✓ Zsh completion loads successfully"
    echo "✓ Completion function '_workspace' is defined"
else
    echo "✗ Failed to load zsh completion or function not found"
fi
echo

# Test install-completion command
echo "=== Testing install-completion command ==="
if ./workspace install-completion &>/dev/null <<< "n"; then
    echo "✓ install-completion command works"
else
    echo "✗ install-completion command failed"
fi
echo

# Check for required files
echo "=== Checking completion files ==="
for file in workspace-completion.bash workspace-completion.zsh; do
    if [[ -f "$file" ]]; then
        echo "✓ $file exists"
    else
        echo "✗ $file not found"
    fi
done
echo

# Test command list from main script
echo "=== Testing command recognition ==="
commands="switch sync status foreach list clean config install-completion help"
for cmd in $commands; do
    if ./workspace help 2>/dev/null | grep -q "$cmd"; then
        echo "✓ Command '$cmd' is documented"
    else
        echo "✗ Command '$cmd' not found in help"
    fi
done

echo
echo "Completion testing complete!"