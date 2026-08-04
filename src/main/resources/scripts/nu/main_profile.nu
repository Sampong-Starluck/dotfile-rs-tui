# Streamlined Nushell functions — updated for Nu 0.113

# --- Navigation -------------------------------------------------------------

# List only files in the current directory
export def ll [] {
    ls | where type == "file"
}

# Jump to the GitHub directory.
# `--env` is REQUIRED: cd just sets $env.PWD, and a plain `def` can't
# mutate the caller's environment, so without it this command does nothing.
export def --env g [] {
    cd ~/Documents/Github
}

# --- Git --------------------------------------------------------------------

# Pull with rebase, defaulting to the current branch when none is given
# What pull strategy git would pick for this branch, from config:
# branch.<name>.rebase  >  pull.rebase  >  pull.ff
def detect-pull-strategy [branch: string] {
    let branch_rebase = git config --get $"branch.($branch).rebase" | complete
    let pull_rebase   = git config --get pull.rebase | complete
    let pull_ff       = git config --get pull.ff | complete

    let rebase_val = if ($branch_rebase.exit_code == 0 and ($branch_rebase.stdout | str trim | is-not-empty)) {
        $branch_rebase.stdout | str trim          # per-branch wins
    } else {
        $pull_rebase.stdout | str trim            # "" when unset
    }

    if $rebase_val in ["true" "interactive" "merges"] {
        "rebase"
    } else if ($pull_ff.stdout | str trim) == "only" {
        "ff"
    } else {
        "merge"
    }
}

export def gpull [
    branch?: string            # branch to pull (defaults to current)
    --strategy (-s): string    # override detection: ff | rebase | merge
] {
    let target = $branch | default (git branch --show-current | str trim)

    let chosen = if $strategy != null { $strategy } else { detect-pull-strategy $target }
    let source = if $strategy != null { "override" } else { "from config" }

    let flag = match $chosen {
        "ff" | "ff-only"      => "--ff-only"
        "rebase"              => "--rebase"
        "merge" | "no-rebase" => "--no-rebase"
        _ => { error make { msg: $"unknown strategy '($chosen)' — use ff, rebase, or merge" } }
    }

    print $"(ansi green_bold)gpull(ansi reset) ($target)  strategy: (ansi cyan)($chosen)(ansi reset) (ansi grey)\(($source)) → git pull ($flag)(ansi reset)"
    git pull $flag origin $target
}

# Show git log, optionally as a decorated graph
export def gmsg [--decorative (-d)] {
    if $decorative {
        git log --graph --oneline --decorate
    } else {
        git log
    }
}

# Stage everything and commit with a message
export def gcom [message: string] {
    git add .
    git commit -m $message
}

# Stage everything and amend the last commit without editing its message
export def gamend [] {
    git add .
    git commit --amend --no-edit
}

# Show git status
export def gstat [] {
    git status
}

# Checkout a branch (branch name is required — Nu enforces it for us)
export def gcheck [branch: string] {
    git checkout $branch
}

# Create-and-switch to a branch, optionally with a type prefix.
# Note: -h is reserved for --help on every custom command, so hotfix uses -x.
export def gcbranch [
    branch?: string
    --bugs (-b)
    --feats (-f)
    --hotfix (-x)
] {
    if $branch == null {
        git branch
        return
    }

    if ($hotfix or $feats or $bugs) {
        let branch_type = if $hotfix { "hotfix" } else if $feats { "feats" } else { "bugs" }
        git checkout -b $"($branch_type)/($branch)"
    } else {
        git checkout $branch
    }
}

# Combined "lazy" add / commit / pull / push workflow
export def lazygcom [
    message?: string
    --amend (-a)
    --branch (-b): string = "development"
    --no-pull (-n)
    --no-push (-p)
] {
    git add .

    if $amend {
        git commit --amend --no-edit
    } else if ($message | is-not-empty) {
        git commit -m $message
    } else {
        error make { msg: "Commit message required when not amending" }
    }

    if not $no_pull { git pull origin $branch }
    if not $no_push { git push }
}

# --- Utilities --------------------------------------------------------------

# Get the public IP address (built-in http get, no external curl needed)
export def get-pubip [] {
    http get https://ifconfig.me/ip | str trim
}

# Load an Oh My Posh theme by file name, with validation.
# To switch themes live, source the output: load_theme "x.omp.json" | save -f ~/.omp.nu; source ~/.omp.nu
export def load_theme [theme: string] {
    let theme_path = $env.POSH_THEMES_PATH | path join $theme
    if not ($theme_path | path exists) {
        error make { msg: $"Theme not found: ($theme_path)" }
    }
    oh-my-posh init nu --config $theme_path
}

# Startup configuration.
# `--env` is REQUIRED here too, or every $env.config edit below is discarded
# the moment this command returns.
export def --env startup [] {
    $env.config.history = {
        file_format: sqlite
        max_size: 1_000_000
        sync_on_enter: true
        isolation: true
    }

    $env.config.show_banner = false

    $env.config.buffer_editor = if (which vim | is-not-empty) { "vim" } else { "code" }
}
