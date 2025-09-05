#!/usr/bin/env bash
# Bash completion for workspace script

_workspace_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Main commands
    local commands="switch sync status foreach list clean config repair install-completion help"
    
    # Helper function to show completion hints
    _show_hint() {
        # Show hints when:
        # - Tab is pressed twice (COMP_TYPE=63 means '?')
        # - Or when bash-completion is available and active
        # This provides helpful context about what values are expected
        if [[ "${COMP_TYPE:-}" == "63" ]] || [[ -n "${BASH_COMPLETION_COMPAT_DIR:-}" ]]; then
            printf "\n%s\n" "$1" >&2
        fi
    }
    
    # Config subcommands
    local config_subcommands="set-default set show help"
    
    # Get workspace script location
    local workspace_script="${COMP_WORDS[0]}"
    local workspace_dir="$(dirname "$(readlink -f "$workspace_script")")"
    local worktrees_dir="$workspace_dir/worktrees"
    
    # Function to get available workspaces
    _get_workspaces() {
        if [[ -d "$worktrees_dir" ]]; then
            for dir in "$worktrees_dir"/*; do
                [[ -d "$dir" ]] && basename "$dir"
            done
        fi
    }
    
    # Function to get repository names from template
    _get_repo_names() {
        local repos
        if [[ -d "$workspace_dir/.git" ]]; then
            repos=$(cd "$workspace_dir" && git config --get-all workspace.repo 2>/dev/null | while read -r line; do
                url="${line%% *}"
                basename "${url%.git}"
            done)
            echo "$repos"
        fi
    }
    
    # Function to get repository URLs from configuration
    _get_repo_urls() {
        local workspace="${1:-}"
        local urls
        
        # First try workspace-specific config if workspace exists
        if [[ -n "$workspace" && -e "$worktrees_dir/$workspace/.git" ]]; then
            urls=$(cd "$worktrees_dir/$workspace" 2>/dev/null && git config --worktree --get-all workspace.repo 2>/dev/null | while read -r line; do
                echo "${line%% *}"
            done)
            if [[ -n "$urls" ]]; then
                echo "$urls"
                return
            fi
        fi
        
        # Fall back to superproject default config
        if [[ -d "$workspace_dir/.git" ]]; then
            urls=$(cd "$workspace_dir" && git config --get-all workspace.repo 2>/dev/null | while read -r line; do
                echo "${line%% *}"
            done)
            echo "$urls"
        fi
    }
    
    # Function to get repository info (URL + current branch/ref)
    _get_repo_info() {
        local workspace="${1:-}"
        local repo_name="${2:-}"
        
        # First try workspace-specific config if workspace exists
        if [[ -n "$workspace" && -e "$worktrees_dir/$workspace/.git" ]]; then
            local config=$(cd "$worktrees_dir/$workspace" 2>/dev/null && git config --worktree --get-all workspace.repo 2>/dev/null)
            if [[ -n "$config" ]]; then
                echo "$config" | while read -r line; do
                    local url="${line%% *}"
                    local name=$(basename "${url%.git}")
                    if [[ -z "$repo_name" || "$name" == "$repo_name" ]]; then
                        echo "$line"
                    fi
                done
                return
            fi
        fi
        
        # Fall back to superproject default config
        if [[ -d "$workspace_dir/.git" ]]; then
            local config=$(cd "$workspace_dir" && git config --get-all workspace.repo 2>/dev/null)
            echo "$config" | while read -r line; do
                local url="${line%% *}"
                local name=$(basename "${url%.git}")
                if [[ -z "$repo_name" || "$name" == "$repo_name" ]]; then
                    echo "$line"
                fi
            done
        fi
    }
    
    # Main command completion
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        _show_hint "workspace commands:"
        COMPREPLY=( $(compgen -W "$commands" -- "$cur") )
        return 0
    fi
    
    # Subcommand and argument completion
    case "${COMP_WORDS[1]}" in
        switch)
            # Complete with available workspace names or suggest new ones
            if [[ ${COMP_CWORD} -eq 2 ]]; then
                local workspaces="$(_get_workspaces)"
                # Add common branch name suggestions
                local suggestions="main master develop feature- hotfix- release-"
                _show_hint "existing workspaces or new branch name:"
                COMPREPLY=( $(compgen -W "$workspaces $suggestions" -- "$cur") )
            fi
            ;;
            
        sync)
            # Complete with existing workspace names
            if [[ ${COMP_CWORD} -eq 2 ]]; then
                local workspaces="$(_get_workspaces)"
                _show_hint "workspace to sync:"
                COMPREPLY=( $(compgen -W "$workspaces" -- "$cur") )
            fi
            ;;
            
        clean)
            # Complete with existing workspace names (required argument)
            if [[ ${COMP_CWORD} -eq 2 ]]; then
                local workspaces="$(_get_workspaces)"
                _show_hint "workspace to remove:"
                COMPREPLY=( $(compgen -W "$workspaces" -- "$cur") )
            fi
            ;;
            
        foreach)
            # Complete with workspace names or commands
            if [[ ${COMP_CWORD} -eq 2 ]]; then
                # First arg could be workspace or command
                local workspaces="$(_get_workspaces)"
                # Common git commands
                local git_cmds="status pull push fetch checkout branch log diff"
                _show_hint "workspace name or command to run:"
                COMPREPLY=( $(compgen -W "$workspaces" -- "$cur") )
                # Also complete with commands if it doesn't look like a workspace
                if [[ ${#COMPREPLY[@]} -eq 0 ]]; then
                    COMPREPLY=( $(compgen -c -- "$cur") )
                fi
            elif [[ ${COMP_CWORD} -eq 3 ]]; then
                # If first arg was a workspace, complete with commands
                local workspaces="$(_get_workspaces)"
                if [[ " $workspaces " =~ " ${COMP_WORDS[2]} " ]]; then
                    _show_hint "command to run in repositories:"
                    COMPREPLY=( $(compgen -c -- "$cur") )
                fi
            fi
            ;;
            
        config)
            if [[ ${COMP_CWORD} -eq 2 ]]; then
                # Complete config subcommands
                _show_hint "workspace config subcommands:"
                COMPREPLY=( $(compgen -W "$config_subcommands" -- "$cur") )
            else
                # Handle config subcommand arguments
                case "${COMP_WORDS[2]}" in
                    set-default)
                        # set-default <url> [branch] [ref]
                        # Can't really complete URLs, but can suggest branches for 2nd arg
                        if [[ ${COMP_CWORD} -eq 3 ]]; then
                            _show_hint "repository URL:"
                        elif [[ ${COMP_CWORD} -eq 4 ]]; then
                            local branches="main master develop"
                            _show_hint "default branch (optional):"
                            COMPREPLY=( $(compgen -W "$branches" -- "$cur") )
                        elif [[ ${COMP_CWORD} -eq 5 ]]; then
                            _show_hint "ref/tag/commit (optional):"
                        fi
                        ;;
                        
                    set)
                        # set <workspace> <url|repo-name> [branch] [ref]
                        if [[ ${COMP_CWORD} -eq 3 ]]; then
                            # Complete workspace names
                            local workspaces="$(_get_workspaces)"
                            _show_hint "workspace name:"
                            COMPREPLY=( $(compgen -W "$workspaces" -- "$cur") )
                        elif [[ ${COMP_CWORD} -eq 4 ]]; then
                            # Complete with URLs from configuration
                            local workspace="${COMP_WORDS[3]}"
                            local urls="$(_get_repo_urls "$workspace")"
                            local repos="$(_get_repo_names)"
                            _show_hint "repository URL (from config) or repo name:"
                            # Combine URLs and repo names for completion
                            COMPREPLY=( $(compgen -W "$urls $repos" -- "$cur") )
                        elif [[ ${COMP_CWORD} -eq 5 ]]; then
                            # For branch completion, check if previous arg was a URL or repo name
                            local workspace="${COMP_WORDS[3]}"
                            local url_or_name="${COMP_WORDS[4]}"
                            local current_branch=""
                            
                            # Try to get current branch/ref for this repository
                            if [[ "$url_or_name" =~ ^https?:// || "$url_or_name" =~ ^git@ || "$url_or_name" =~ ^ssh:// || "$url_or_name" =~ \.git$ ]]; then
                                # It's a URL, find current branch for this URL
                                local repo_info="$(_get_repo_info "$workspace")"
                                while read -r line; do
                                    local url="${line%% *}"
                                    if [[ "$url" == "$url_or_name" ]]; then
                                        local parts=($line)
                                        current_branch="${parts[1]:-}"
                                        break
                                    fi
                                done <<< "$repo_info"
                            else
                                # It's a repo name, find current branch for this repo
                                local repo_info="$(_get_repo_info "$workspace" "$url_or_name")"
                                if [[ -n "$repo_info" ]]; then
                                    local parts=($repo_info)
                                    current_branch="${parts[1]:-}"
                                fi
                            fi
                            
                            # Suggest current branch first, then common branches
                            local branches="$current_branch main master develop feature- hotfix- release-"
                            _show_hint "branch name (current: ${current_branch:-none}):"
                            COMPREPLY=( $(compgen -W "$branches" -- "$cur") )
                        elif [[ ${COMP_CWORD} -eq 6 ]]; then
                            _show_hint "ref/tag/commit (optional):"
                        fi
                        ;;
                        
                    show)
                        # show [workspace]
                        if [[ ${COMP_CWORD} -eq 3 ]]; then
                            local workspaces="$(_get_workspaces)"
                            _show_hint "workspace name (optional):"
                            COMPREPLY=( $(compgen -W "$workspaces" -- "$cur") )
                        fi
                        ;;
                esac
            fi
            ;;
            
        repair)
            # repair <workspace> <repository-name>
            if [[ ${COMP_CWORD} -eq 2 ]]; then
                # Complete workspace names
                local workspaces="$(_get_workspaces)"
                _show_hint "workspace name:"
                COMPREPLY=( $(compgen -W "$workspaces" -- "$cur") )
            elif [[ ${COMP_CWORD} -eq 3 ]]; then
                # Complete repository names
                local repos="$(_get_repo_names)"
                _show_hint "repository name:"
                COMPREPLY=( $(compgen -W "$repos" -- "$cur") )
            fi
            ;;
            
        install-completion)
            # install-completion <shell>
            if [[ ${COMP_CWORD} -eq 2 ]]; then
                local shells="bash zsh"
                _show_hint "shell type:"
                COMPREPLY=( $(compgen -W "$shells" -- "$cur") )
            fi
            ;;
            
        status|list|help)
            # These commands don't take arguments
            ;;
            
        *)
            # Unknown command, no completion
            ;;
    esac
}

# Register completion for the workspace script
complete -F _workspace_completions workspace

# Also register for direct script invocation
if [[ -n "${BASH_SOURCE[0]}" ]]; then
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [[ -f "$script_dir/workspace" ]]; then
        complete -F _workspace_completions "$script_dir/workspace"
        complete -F _workspace_completions ./workspace
    fi
fi