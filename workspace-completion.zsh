#compdef workspace
# Zsh completion for workspace script
#
# To remove the "Completing" prefix from completion descriptions,
# add this to your .zshrc after loading completions:
#   zstyle ':completion:*:descriptions' format '%B%d:%b'
# Or to remove descriptions entirely:
#   zstyle ':completion:*:descriptions' format ''

_workspace() {
    local context state line
    local workspace_dir worktrees_dir
    
    # Get workspace script location
    # Save the original command before _arguments modifies $words
    local orig_cmd="${words[1]}"
    
    # Determine workspace directory properly
    # If we're completing ./workspace or /path/to/workspace, use that
    # Otherwise, if just "workspace", try to find it
    if [[ "$orig_cmd" == */* ]]; then
        workspace_dir="$(dirname "$(readlink -f "$orig_cmd")")"
    else
        # Assume we're in the workspace directory or it's in PATH
        if [[ -x "./workspace" ]]; then
            workspace_dir="$(dirname "$(readlink -f "./workspace")")"
        elif command -v workspace >/dev/null 2>&1; then
            workspace_dir="$(dirname "$(readlink -f "$(command -v workspace)")")"
        else
            workspace_dir="$PWD"
        fi
    fi
    worktrees_dir="$workspace_dir/worktrees"
    
    # Function to get available workspaces
    _get_workspaces() {
        local workspaces=()
        if [[ -d "$worktrees_dir" ]]; then
            for dir in "$worktrees_dir"/*(/N); do
                workspaces+=(${dir:t})
            done
        fi
        echo "${workspaces[@]}"
    }
    
    # Function to get repository names from template
    _get_repo_names() {
        local repos=()
        if [[ -d "$workspace_dir/.git" ]]; then
            while IFS= read -r line; do
                [[ -n "$line" ]] || continue
                local url="${line%% *}"
                repos+=(${url:t:r})  # basename without .git extension
            done < <(cd "$workspace_dir" && git config --get-all workspace.repo 2>/dev/null)
        fi
        echo "${repos[@]}"
    }
    
    # Function to get repository URLs from configuration
    _get_repo_urls() {
        local workspace="${1:-}"
        local urls=()
        
        # First try workspace-specific config if workspace exists
        if [[ -n "$workspace" && -e "$worktrees_dir/$workspace/.git" ]]; then
            while IFS= read -r line; do
                [[ -n "$line" ]] || continue
                urls+=(${line%% *})
            done < <(cd "$worktrees_dir/$workspace" 2>/dev/null && git config --worktree --get-all workspace.repo 2>/dev/null)
            if (( ${#urls[@]} > 0 )); then
                echo "${urls[@]}"
                return
            fi
        fi
        
        # Fall back to superproject default config
        if [[ -d "$workspace_dir/.git" ]]; then
            while IFS= read -r line; do
                [[ -n "$line" ]] || continue
                urls+=(${line%% *})
            done < <(cd "$workspace_dir" && git config --get-all workspace.repo 2>/dev/null)
        fi
        echo "${urls[@]}"
    }
    
    # Function to get repository info (URL + current branch/ref)
    _get_repo_info() {
        local workspace="${1:-}"
        local repo_name="${2:-}"
        local info_lines=()
        
        # First try workspace-specific config if workspace exists
        if [[ -n "$workspace" && -e "$worktrees_dir/$workspace/.git" ]]; then
            while IFS= read -r line; do
                [[ -n "$line" ]] || continue
                local url="${line%% *}"
                local name=${url:t:r}  # basename without .git extension
                if [[ -z "$repo_name" || "$name" == "$repo_name" ]]; then
                    info_lines+=("$line")
                fi
            done < <(cd "$worktrees_dir/$workspace" 2>/dev/null && git config --worktree --get-all workspace.repo 2>/dev/null)
            if (( ${#info_lines[@]} > 0 )); then
                printf '%s\n' "${info_lines[@]}"
                return
            fi
        fi
        
        # Fall back to superproject default config
        if [[ -d "$workspace_dir/.git" ]]; then
            while IFS= read -r line; do
                [[ -n "$line" ]] || continue
                local url="${line%% *}"
                local name=${url:t:r}  # basename without .git extension
                if [[ -z "$repo_name" || "$name" == "$repo_name" ]]; then
                    info_lines+=("$line")
                fi
            done < <(cd "$workspace_dir" && git config --get-all workspace.repo 2>/dev/null)
            printf '%s\n' "${info_lines[@]}"
        fi
    }
    
    # Main command definitions
    local -a commands
    commands=(
        'switch:Switch to workspace (create if needed)'
        'sync:Update repositories in workspace'
        'status:Show workspace and repository status'
        'foreach:Execute command in all repositories'
        'list:List available workspaces'
        'clean:Remove a workspace'
        'config:Manage workspace configuration'
        'repair:Repair broken repository'
        'install-completion:Install shell completion'
        'help:Show help information'
    )
    
    # Config subcommand definitions
    local -a config_subcommands
    config_subcommands=(
        'set-default:Add repository to template configuration'
        'set:Set workspace-specific repository branch/ref'
        'show:Display workspace configuration'
        'help:Show configuration help'
    )
    
    _arguments -C \
        '1: :->command' \
        '*:: :->args'
    
    case $state in
        command)
            _describe -t commands 'workspace commands' commands
            ;;
            
        args)
            case $words[1] in
                switch)
                    if [[ $CURRENT -eq 2 ]]; then
                        local -a workspaces suggestions
                        workspaces=($(_get_workspaces))
                        suggestions=(main master develop feature- hotfix- release-)
                        _alternative \
                            "workspaces:existing workspaces or new branch name:($workspaces)" \
                            "branches:branch suggestions:($suggestions)"
                    fi
                    ;;
                    
                sync)
                    if [[ $CURRENT -eq 2 ]]; then
                        local -a workspaces
                        workspaces=($(_get_workspaces))
                        _describe -t workspaces 'workspace to sync' workspaces
                    fi
                    ;;
                    
                clean)
                    if [[ $CURRENT -eq 2 ]]; then
                        local -a workspaces
                        workspaces=($(_get_workspaces))
                        _describe -t workspaces 'workspace to remove' workspaces
                    fi
                    ;;
                    
                foreach)
                    if [[ $CURRENT -eq 2 ]]; then
                        # Could be workspace or command
                        local -a workspaces
                        workspaces=($(_get_workspaces))
                        _alternative \
                            "workspaces:workspace name or command to run:($workspaces)" \
                            'commands:command to run:_command_names'
                    elif [[ $CURRENT -eq 3 ]]; then
                        # If previous was a workspace, complete with commands
                        local -a workspaces
                        workspaces=($(_get_workspaces))
                        if (( ${workspaces[(I)$words[2]]} )); then
                            _message 'command to run in repositories'
                            _command_names
                        else
                            # Previous was a command, complete command arguments
                            _normal
                        fi
                    else
                        _normal
                    fi
                    ;;
                    
                config)
                    if [[ $CURRENT -eq 2 ]]; then
                        _describe -t config-commands 'workspace config subcommands' config_subcommands
                    else
                        case $words[2] in
                            set-default)
                                case $CURRENT in
                                    3)
                                        _message 'repository URL'
                                        ;;
                                    4)
                                        local -a branches
                                        branches=(main master develop)
                                        _describe -t branches 'default branch (optional)' branches
                                        ;;
                                    5)
                                        _message 'ref/tag/commit (optional)'
                                        ;;
                                esac
                                ;;
                                
                            set)
                                case $CURRENT in
                                    3)
                                        local -a workspaces
                                        workspaces=($(_get_workspaces))
                                        _describe -t workspaces 'workspace name' workspaces
                                        ;;
                                    4)
                                        # Complete with URLs from configuration and repo names
                                        local workspace="$words[3]"
                                        local -a urls repos combined
                                        urls=($(_get_repo_urls "$workspace"))
                                        repos=($(_get_repo_names))
                                        combined=("${urls[@]}" "${repos[@]}")
                                        _alternative \
                                            'urls:repository URL from config:('"${(j: :)urls}"')' \
                                            'repos:repository name:('"${(j: :)repos}"')'
                                        ;;
                                    5)
                                        # For branch completion, get current branch for context
                                        local workspace="$words[3]"
                                        local url_or_name="$words[4]"
                                        local current_branch=""
                                        local -a branches
                                        
                                        # Try to get current branch/ref for this repository
                                        local repo_info="$(_get_repo_info "$workspace" "$url_or_name")"
                                        if [[ -n "$repo_info" ]]; then
                                            local parts=($=repo_info)  # zsh array splitting
                                            current_branch="${parts[2]:-}"
                                        fi
                                        
                                        # Include current branch if found, plus common branches
                                        branches=($current_branch main master develop feature- hotfix- release-)
                                        _describe -t branches "branch name (current: ${current_branch:-none})" branches
                                        ;;
                                    6)
                                        _message 'ref/tag/commit (optional)'
                                        ;;
                                esac
                                ;;
                                
                            show)
                                if [[ $CURRENT -eq 3 ]]; then
                                    local -a workspaces
                                    workspaces=($(_get_workspaces))
                                    _describe -t workspaces 'workspace name (optional)' workspaces
                                fi
                                ;;
                                
                            help)
                                # No arguments
                                ;;
                        esac
                    fi
                    ;;
                    
                repair)
                    if [[ $CURRENT -eq 2 ]]; then
                        local -a workspaces
                        workspaces=($(_get_workspaces))
                        _describe -t workspaces 'workspace name' workspaces
                    elif [[ $CURRENT -eq 3 ]]; then
                        local -a repos
                        repos=($(_get_repo_names))
                        _describe -t repos 'repository name' repos
                    fi
                    ;;
                    
                install-completion)
                    if [[ $CURRENT -eq 2 ]]; then
                        local -a shells
                        shells=(bash zsh)
                        _describe -t shells 'shell type' shells
                    fi
                    ;;
                    
                status|list|help)
                    # These commands don't take arguments
                    ;;
            esac
            ;;
    esac
}

# Register completion
_workspace "$@"

# Also set up for direct invocation
if [[ -n "${(%):-%x}" ]]; then
    local script_dir="${${(%):-%x}:A:h}"
    if [[ -f "$script_dir/workspace" ]]; then
        compdef _workspace "$script_dir/workspace"
        compdef _workspace ./workspace
    fi
fi