######################################################################
#
#
#           ██████╗  █████╗ ███████╗██╗  ██╗██████╗  ██████╗
#           ██╔══██╗██╔══██╗██╔════╝██║  ██║██╔══██╗██╔════╝
#           ██████╔╝███████║███████╗███████║██████╔╝██║
#           ██╔══██╗██╔══██║╚════██║██╔══██║██╔══██╗██║
#           ██████╔╝██║  ██║███████║██║  ██║██║  ██║╚██████╗
#           ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝
#
#
######################################################################

set -o vi

# --- History ----------------------------------------------------------------
HISTTIMEFORMAT="%F %T "
HISTCONTROL=ignoredups
HISTSIZE=2000
HISTFILESIZE=2000
shopt -s histappend

# --- Aliases ----------------------------------------------------------------
alias cls='clear'
alias tree='cmd //c tree //a //f'

alias gs='git status'
alias ga='git add'
alias gaa='git add --all'
alias gc='git commit'
alias gl='git log --oneline'
alias gb='git checkout -b'
alias gd='git diff'

alias ..='cd ..;pwd'
alias ...='cd ../..;pwd'
alias ....='cd ../../..;pwd'

alias c='clear'
alias h='history'
alias mkdir='mkdir -p -v'

# --- Navigation / listing (ported from Nushell) -----------------------------

# List only regular files in the current directory (mirrors nu's
# `ls | where type == "file"`; excludes dirs, symlinks, and hidden files).
unalias ll 2>/dev/null || true
ll() {
    ls -lh "$@" | grep '^-'
}

# Jump to the GitHub directory. (In bash, cd inside a function affects the
# current shell directly — no nu-style `--env` needed.)
g() {
    cd ~/Documents/Github || return
}

# --- Git workflow functions (ported from Nushell) ---------------------------

# Pull with rebase, defaulting to the current branch when none is given.
gpull() {
    local branch="${1:-$(git branch --show-current)}"
    git pull origin "$branch" --rebase
}

# Show git log; pass -d / --decorative for a decorated graph view.
gmsg() {
    if [[ "$1" == "-d" || "$1" == "--decorative" ]]; then
        git log --graph --oneline --decorate
    else
        git log
    fi
}

# Stage everything and commit with a message (message required).
gcom() {
    if [ -z "$1" ]; then
        echo "gcom: commit message required" >&2
        return 1
    fi
    git add .
    git commit -m "$1"
}

# Stage everything and amend the last commit without editing its message.
gamend() {
    git add .
    git commit --amend --no-edit
}

# Show git status.
gstat() {
    git status
}

# Checkout a branch (branch name required).
gcheck() {
    if [ -z "$1" ]; then
        echo "gcheck: branch name required" >&2
        return 1
    fi
    git checkout "$1"
}

# Create-and-switch to a branch, optionally with a type prefix.
# With no argument it just lists branches.
#   gcbranch            -> list branches
#   gcbranch login      -> checkout existing 'login'
#   gcbranch login -f   -> create + switch to 'feats/login'
# Flags: -b/--bugs, -f/--feats, -x/--hotfix  (last one wins if several given)
gcbranch() {
    local prefix="" branch=""
    while [ $# -gt 0 ]; do
        case "$1" in
            -b|--bugs)   prefix="bugs" ;;
            -f|--feats)  prefix="feats" ;;
            -x|--hotfix) prefix="hotfix" ;;
            -*)          echo "gcbranch: unknown flag $1" >&2; return 1 ;;
            *)           branch="$1" ;;
        esac
        shift
    done

    if [ -z "$branch" ]; then
        git branch
        return
    fi

    if [ -n "$prefix" ]; then
        git checkout -b "$prefix/$branch"
    else
        git checkout "$branch"
    fi
}

# Combined "lazy" add / commit / pull / push workflow.
#   lazygcom "fix login bug"     -> add, commit, pull origin development, push
#   lazygcom -a                  -> add, amend, pull, push
#   lazygcom "msg" -b main -n    -> commit, skip pull, push (branch main)
# Flags: -a/--amend, -b/--branch <name> (default: development),
#        -n/--no-pull, -p/--no-push
lazygcom() {
    local message="" branch="development"
    local amend=0 no_pull=0 no_push=0

    while [ $# -gt 0 ]; do
        case "$1" in
            -a|--amend)   amend=1 ;;
            -n|--no-pull) no_pull=1 ;;
            -p|--no-push) no_push=1 ;;
            -b|--branch)
                if [ -z "$2" ]; then
                    echo "lazygcom: --branch requires a name" >&2
                    return 1
                fi
                branch="$2"; shift ;;
            -*)           echo "lazygcom: unknown flag $1" >&2; return 1 ;;
            *)            message="$1" ;;
        esac
        shift
    done

    git add .

    if [ "$amend" -eq 1 ]; then
        git commit --amend --no-edit
    elif [ -n "$message" ]; then
        git commit -m "$message"
    else
        echo "lazygcom: commit message required when not amending" >&2
        return 1
    fi

    [ "$no_pull" -eq 0 ] && git pull origin "$branch"
    [ "$no_push" -eq 0 ] && git push
}

# --- Utility functions ------------------------------------------------------

# Get the public IP address (ported from nu's `http get` -> curl here).
get-pubip() {
    curl -fsS https://ifconfig.me/ip && echo
}

# Search shell history
hg() {
    history | grep "$1"
}

# Show the 20 largest files/dirs in the current directory
find_largest_files() {
    du -h -x -s -- * | sort -r -h | head -20
}

# Create a directory, cd into it, and scaffold a new git repo
git_init() {
    if [ -z "$1" ]; then
        printf "%s\n" "Please provide a directory name."
    else
        mkdir "$1"
        builtin cd "$1" || return
        pwd
        git init
        touch readme.md .gitignore LICENSE
        echo "# $(basename "$PWD")" >> readme.md
    fi
}

# --- Lazy/cached tool init --------------------------------------------------
# The slowest part of shell startup is regenerating init/completion scripts on
# every launch. `ng completion script` is the worst offender: it boots a full
# Node.js process (painfully slow on Git Bash/Windows). We cache that static
# output to a file and just source the cache instead.

# Source the cached stdout of a slow command, regenerating the cache only when
# it's missing/empty or the tool's binary is newer than the cache.
# Usage: _load_cached <cache-name> <command> [args...]
_load_cached() {
    local name="$1"; shift
    local cmd="$1"
    local cache_dir="${XDG_CACHE_HOME:-$HOME/.cache}/bashrc"
    local cache_file="$cache_dir/$name"

    command -v "$cmd" >/dev/null 2>&1 || return 0

    local bin; bin="$(command -v "$cmd")"
    if [[ ! -s "$cache_file" || "$bin" -nt "$cache_file" ]]; then
        mkdir -p "$cache_dir"
        "$@" > "$cache_file" 2>/dev/null
    fi
    # shellcheck disable=SC1090
    source "$cache_file"
}

# Force-rebuild the caches. Run this after upgrading the Angular CLI, changing
# your oh-my-posh theme path, etc.
refresh-shell-cache() {
    rm -rf "${XDG_CACHE_HOME:-$HOME/.cache}/bashrc"
    echo "Shell cache cleared. Run: source ~/.bashrc"
}

# --- Oh My Posh -------------------------------------------------------------
export POSH_THEMES_DIR="$HOME/AppData/Local/Programs/oh-my-posh/themes"
export POSH_THEME="$POSH_THEMES_DIR/darkblood.omp.json"

# Default prompt (cached). The init output only embeds the config *path*, so
# editing the theme file itself does NOT require a cache refresh.
_load_cached oh-my-posh.bash oh-my-posh init bash --config "$POSH_THEME"

# Switch theme live by file name (ported from nu's load_theme, with validation).
# Bypasses the startup cache on purpose so the change applies immediately.
#   load_theme jandedobbeleer.omp.json
load_theme() {
    if [ -z "$1" ]; then
        echo "load_theme: theme file name required" >&2
        return 1
    fi
    local theme_path="$POSH_THEMES_DIR/$1"
    if [ ! -f "$theme_path" ]; then
        echo "load_theme: theme not found: $theme_path" >&2
        return 1
    fi
    eval "$(oh-my-posh init bash --config "$theme_path")"
}

# --- Other tool init --------------------------------------------------------

# Angular CLI completion (cached — this was the main startup bottleneck).
_load_cached ng-completion.bash ng completion script

# vfox (version manager): intentionally NOT cached. Its activation output does
# live PATH manipulation and can contain session/PID-specific paths, so a cached
# copy could bake in a stale PATH. It's a fast Go binary, so eval it directly.
command -v vfox >/dev/null 2>&1 && eval "$(vfox activate bash)"\

eval "$(oh-my-posh init bash --config /usr/share/oh-my-posh/themes/darkblood.omp.json)"

# --- Startup banner ---------------------------------------------------------
printf "\n"
printf "   %s\n" "DATE: $(date)"
printf "\n"
