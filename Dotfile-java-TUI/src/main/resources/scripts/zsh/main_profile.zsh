bindkey -v

# --- History ----------------------------------------------------------------
HISTFILE="$HOME/.zsh_history"
HISTSIZE=2000
SAVEHIST=2000
setopt HIST_IGNORE_DUPS HIST_IGNORE_SPACE SHARE_HISTORY APPEND_HISTORY

# --- Aliases ----------------------------------------------------------------
alias cls='clear'
alias c='clear'
alias h='history'
alias mkdir='mkdir -p -v'

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

# --- Navigation / listing ---------------------------------------------------

ll() {
    ls -lh "$@" | grep '^-'
}

g() {
    cd ~/Documents/Github || return
}

# --- Git workflow functions -------------------------------------------------

gpull() {
    local branch="${1:-$(git branch --show-current)}"
    git pull origin "$branch" --rebase
}

gmsg() {
    if [[ "$1" == "-d" || "$1" == "--decorative" ]]; then
        git log --graph --oneline --decorate
    else
        git log
    fi
}

gcom() {
    if [ -z "$1" ]; then
        echo "gcom: commit message required" >&2
        return 1
    fi
    git add .
    git commit -m "$1"
}

gamend() {
    git add .
    git commit --amend --no-edit
}

gstat() {
    git status
}

gcheck() {
    if [ -z "$1" ]; then
        echo "gcheck: branch name required" >&2
        return 1
    fi
    git checkout "$1"
}

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
    if [ -z "$branch" ]; then git branch; return; fi
    if [ -n "$prefix" ]; then
        git checkout -b "$prefix/$branch"
    else
        git checkout "$branch"
    fi
}

lazygcom() {
    local message="" branch="development"
    local amend=0 no_pull=0 no_push=0
    while [ $# -gt 0 ]; do
        case "$1" in
            -a|--amend)   amend=1 ;;
            -n|--no-pull) no_pull=1 ;;
            -p|--no-push) no_push=1 ;;
            -b|--branch)  branch="$2"; shift ;;
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

get-pubip() {
    curl -fsS https://ifconfig.me/ip && echo
}

hg() {
    history | grep "$1"
}

find_largest_files() {
    du -h -x -s -- * | sort -r -h | head -20
}

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

# --- Oh My Posh -------------------------------------------------------------
export POSH_THEMES_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/oh-my-posh/themes"
export POSH_THEME="$POSH_THEMES_DIR/darkblood.omp.json"

if command -v oh-my-posh >/dev/null 2>&1; then
    eval "$(oh-my-posh init zsh --config "$POSH_THEME")"
fi

load_theme() {
    if [ -z "$1" ]; then echo "load_theme: theme name required" >&2; return 1; fi
    local theme_path="$POSH_THEMES_DIR/$1"
    if [ ! -f "$theme_path" ]; then echo "load_theme: not found: $theme_path" >&2; return 1; fi
    eval "$(oh-my-posh init zsh --config "$theme_path")"
}

# --- Other tool init --------------------------------------------------------
command -v vfox >/dev/null 2>&1 && eval "$(vfox activate zsh)"

# --- Startup banner ---------------------------------------------------------
printf "\n"
printf "   %s\n" "DATE: $(date)"
printf "\n"