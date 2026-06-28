# --- Vi key bindings --------------------------------------------------------
set -g fish_key_bindings fish_vi_key_bindings

# --- Aliases ----------------------------------------------------------------
alias cls='clear'
alias c='clear'
alias mkdir='mkdir -p -v'

alias gs='git status'
alias ga='git add'
alias gaa='git add --all'
alias gc='git commit'
alias gl='git log --oneline'
alias gb='git checkout -b'
alias gd='git diff'

# --- Navigation -------------------------------------------------------------

function ll
    ls -lh $argv | grep '^-'
end

function g
    cd ~/Documents/Github
end

# --- Git workflow functions -------------------------------------------------

function gpull
    set branch (git branch --show-current 2>/dev/null | string trim)
    git pull origin $branch --rebase
end

function gmsg
    if contains -- -d $argv; or contains -- --decorative $argv
        git log --graph --oneline --decorate
    else
        git log
    end
end

function gcom
    if test (count $argv) -eq 0
        echo "gcom: commit message required" >&2
        return 1
    end
    git add .
    git commit -m $argv[1]
end

function gamend
    git add .
    git commit --amend --no-edit
end

function gstat
    git status
end

function gcheck
    if test (count $argv) -eq 0
        echo "gcheck: branch name required" >&2
        return 1
    end
    git checkout $argv[1]
end

function gcbranch
    argparse b/bugs f/feats x/hotfix -- $argv
    or return

    if test (count $argv) -eq 0
        git branch
        return
    end

    set branch $argv[1]

    if set -q _flag_bugs
        git checkout -b "bugs/$branch"
    else if set -q _flag_feats
        git checkout -b "feats/$branch"
    else if set -q _flag_hotfix
        git checkout -b "hotfix/$branch"
    else
        git checkout $branch
    end
end

function lazygcom
    argparse a/amend n/no-pull p/no-push 'b/branch=' -- $argv
    or return

    set message (count $argv -gt 0 and echo $argv[1])

    git add .

    if set -q _flag_amend
        git commit --amend --no-edit
    else if test -n "$message"
        git commit -m $message
    else
        echo "lazygcom: commit message required when not amending" >&2
        return 1
    end

    set branch (set -q _flag_branch; and echo $_flag_branch; or echo "development")
    if not set -q _flag_no_pull
        git pull origin $branch
    end
    if not set -q _flag_no_push
        git push
    end
end

# --- Utility functions ------------------------------------------------------

function get-pubip
    curl -fsS https://ifconfig.me/ip && echo
end

function hg
    history search $argv[1]
end

function find_largest_files
    du -h -x -s -- * | sort -r -h | head -20
end

function git_init
    if test (count $argv) -eq 0
        echo "Please provide a directory name."
        return 1
    end
    mkdir -p $argv[1]
    cd $argv[1]
    git init
    touch readme.md .gitignore LICENSE
    echo "# "(basename (pwd)) >> readme.md
end

# --- Oh My Posh -------------------------------------------------------------
set -gx POSH_THEMES_DIR (string join / (set -q XDG_DATA_HOME; and echo $XDG_DATA_HOME; or echo "$HOME/.local/share") oh-my-posh themes)
set -gx POSH_THEME "$POSH_THEMES_DIR/darkblood.omp.json"

if command -v oh-my-posh >/dev/null 2>&1
    oh-my-posh init fish --config $POSH_THEME | source
end

function load_theme
    if test (count $argv) -eq 0
        echo "load_theme: theme name required" >&2
        return 1
    end
    set theme_path "$POSH_THEMES_DIR/$argv[1]"
    if not test -f $theme_path
        echo "load_theme: not found: $theme_path" >&2
        return 1
    end
    oh-my-posh init fish --config $theme_path | source
end

# --- Other tool init --------------------------------------------------------
if command -v vfox >/dev/null 2>&1
    vfox activate fish | source
end

# --- Startup banner ---------------------------------------------------------
echo ""
echo "   DATE: "(date)
echo ""