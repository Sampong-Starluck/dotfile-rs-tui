use std::{io::Write, path::PathBuf};
use serde::{Deserialize, Serialize};
use crate::models::{ShellConfig, ShellEntry};

const SHELLS_JSON:  &str = include_str!("../json/shells.json");
const BASH_SCRIPT:  &str = include_str!("../scripts/bash/main_profile.sh");
const NU_SCRIPT:    &str = include_str!("../scripts/nu/main_profile.nu");
const POSH_SCRIPT:  &str = include_str!("../scripts/posh/main_profile.ps1");
const ZSH_SCRIPT:   &str = include_str!("../scripts/zsh/main_profile.zsh");
const FISH_SCRIPT:  &str = include_str!("../scripts/fish/main_profile.fish");

// ── Shell status ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ShellStatus {
    pub entry: ShellEntry,
    pub detected: bool,
    pub deployed: bool,
    pub target_path: Option<PathBuf>,
}

pub fn load_shell_statuses() -> Vec<ShellStatus> {
    read_shells()
        .into_iter()
        .map(|entry| {
            let detected = is_shell_detected(&entry.id);
            let target   = script_target(&entry.id);
            let deployed = target.as_ref().map(|p| p.exists()).unwrap_or(false);
            ShellStatus { detected, deployed, target_path: target, entry }
        })
        .collect()
}

pub fn read_shells() -> Vec<ShellEntry> {
    let config: ShellConfig = serde_json::from_str(SHELLS_JSON)
        .unwrap_or(ShellConfig { shells: vec![] });
    let mut shells: Vec<ShellEntry> = config.shells
        .into_iter()
        .filter(|s| !s.hidden && has_script(&s.id))
        .collect();
    shells.sort_by_key(|s| s.order);
    shells
}

// ── Shell detection ───────────────────────────────────────────────────────────

pub fn shell_binary(id: &str) -> Option<&'static str> {
    match id {
        "bash"       => Some("bash"),
        "zsh"        => Some("zsh"),
        "fish"       => Some("fish"),
        "nushell"    => Some("nu"),
        "powershell" => Some("pwsh"),
        _ => None,
    }
}

pub fn is_shell_detected(id: &str) -> bool {
    shell_binary(id)
        .map(|bin| which::which(bin).is_ok())
        .unwrap_or(false)
}

/// Returns the `chsh -s <path>` command string needed to make the shell the
/// system default, or `None` on Windows (where chsh doesn't apply).
pub fn chsh_command(shell_id: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    { return None; }
    #[cfg(not(target_os = "windows"))]
    {
        let bin  = shell_binary(shell_id)?;
        let path = which::which(bin).ok()?;
        Some(format!("chsh -s {}", path.display()))
    }
}

/// Reads $SHELL (Linux/macOS) or checks for pwsh (Windows) to find the
/// system's current default shell. Returns the shell id string.
pub fn detect_default_shell() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        if which::which("pwsh").is_ok() {
            return Some("powershell".into());
        }
        return None;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let path = std::env::var("SHELL").ok()?;
        let name = path.rsplit('/').next()?;
        match name {
            "bash"                  => Some("bash".into()),
            "zsh"                   => Some("zsh".into()),
            "fish"                  => Some("fish".into()),
            "nu"                    => Some("nushell".into()),
            "pwsh" | "powershell"   => Some("powershell".into()),
            _ => None,
        }
    }
}

// ── Paths ─────────────────────────────────────────────────────────────────────

/// XDG_DATA_HOME/dotfile-rs/scripts  (Linux/macOS)
/// %APPDATA%\dotfile-rs\scripts      (Windows)
pub fn scripts_base_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir().join("AppData").join("Roaming"));
        base.join("dotfile-rs").join("scripts")
    }
    #[cfg(not(target_os = "windows"))]
    {
        let xdg = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir().join(".local").join("share"));
        xdg.join("dotfile-rs").join("scripts")
    }
}

/// XDG_CONFIG_HOME/dotfile-rs  (Linux/macOS)
/// %APPDATA%\dotfile-rs        (Windows)
fn config_base_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir().join("AppData").join("Roaming"));
        base.join("dotfile-rs")
    }
    #[cfg(not(target_os = "windows"))]
    {
        let xdg = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir().join(".config"));
        xdg.join("dotfile-rs")
    }
}

pub fn script_target(shell_id: &str) -> Option<PathBuf> {
    let base = scripts_base_dir();
    match shell_id {
        "bash"       => Some(base.join("bash").join("main_profile.sh")),
        "zsh"        => Some(base.join("zsh").join("main_profile.zsh")),
        "fish"       => Some(base.join("fish").join("main_profile.fish")),
        "nushell"    => Some(base.join("nu").join("main_profile.nu")),
        "powershell" => Some(base.join("posh").join("main_profile.ps1")),
        _ => None,
    }
}

/// The shell's startup profile where we add/remove the source line.
pub fn shell_profile_path(shell_id: &str) -> Option<PathBuf> {
    match shell_id {
        "bash" => {
            if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
                Some(home_dir().join(".bash_profile"))
            } else {
                Some(home_dir().join(".bashrc"))
            }
        }
        "zsh" => Some(home_dir().join(".zshrc")),
        "fish" => {
            if cfg!(target_os = "windows") {
                let base = std::env::var("APPDATA")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| home_dir().join("AppData").join("Roaming"));
                Some(base.join("fish").join("config.fish"))
            } else {
                let xdg = std::env::var("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| home_dir().join(".config"));
                Some(xdg.join("fish").join("config.fish"))
            }
        }
        "nushell" => {
            if cfg!(target_os = "windows") {
                let base = std::env::var("APPDATA")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| home_dir().join("AppData").join("Roaming"));
                Some(base.join("nushell").join("config.nu"))
            } else {
                let xdg = std::env::var("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| home_dir().join(".config"));
                Some(xdg.join("nushell").join("config.nu"))
            }
        }
        "powershell" => {
            if cfg!(target_os = "windows") {
                Some(home_dir().join("Documents").join("PowerShell").join("Microsoft.PowerShell_profile.ps1"))
            } else {
                let xdg = std::env::var("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| home_dir().join(".config"));
                Some(xdg.join("powershell").join("Microsoft.PowerShell_profile.ps1"))
            }
        }
        _ => None,
    }
}

// ── Source line ───────────────────────────────────────────────────────────────

pub fn source_hint(shell_id: &str) -> Option<String> {
    let target = script_target(shell_id)?;
    let path = target.display().to_string();
    let line = match shell_id {
        "bash" | "zsh" | "fish" => format!("source \"{}\"", path),
        "nushell"                => format!("source \"{}\"", path),
        "powershell"             => format!(". \"{}\"", path),
        _ => return None,
    };
    Some(line)
}

// ── Deploy / undeploy ─────────────────────────────────────────────────────────

pub fn deploy_script(shell_id: &str) -> Result<PathBuf, String> {
    let target  = script_target(shell_id)
        .ok_or_else(|| format!("no script for shell '{}'", shell_id))?;
    let content = script_content(shell_id)
        .ok_or_else(|| format!("no script content for '{}'", shell_id))?;

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create dir: {}", e))?;
    }
    std::fs::write(&target, content)
        .map_err(|e| format!("write script: {}", e))?;
    Ok(target)
}

pub fn undeploy_script(shell_id: &str) -> Result<(), String> {
    let target = script_target(shell_id)
        .ok_or_else(|| format!("no script for shell '{}'", shell_id))?;
    if target.exists() {
        std::fs::remove_file(&target)
            .map_err(|e| format!("remove file: {}", e))?;
    }
    Ok(())
}

// ── Profile add / remove ──────────────────────────────────────────────────────

/// Returns `Ok(true)` when the line was added, `Ok(false)` when already present.
pub fn add_source_to_profile(shell_id: &str) -> Result<(bool, PathBuf), String> {
    let profile     = shell_profile_path(shell_id)
        .ok_or_else(|| format!("no profile path for '{}'", shell_id))?;
    let source_line = source_hint(shell_id)
        .ok_or_else(|| format!("no source hint for '{}'", shell_id))?;

    if let Some(parent) = profile.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create profile dir: {}", e))?;
    }

    let existing = if profile.exists() {
        std::fs::read_to_string(&profile).map_err(|e| format!("read profile: {}", e))?
    } else {
        String::new()
    };

    if existing.lines().any(|l| l.trim() == source_line.trim()) {
        return Ok((false, profile));
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&profile)
        .map_err(|e| format!("open profile: {}", e))?;

    writeln!(file, "\n# dotfile-rs\n{}", source_line)
        .map_err(|e| format!("write profile: {}", e))?;

    Ok((true, profile))
}

/// Returns `Ok(true)` when something was removed, `Ok(false)` when nothing to remove.
pub fn remove_source_from_profile(shell_id: &str) -> Result<(bool, PathBuf), String> {
    let profile = shell_profile_path(shell_id)
        .ok_or_else(|| format!("no profile path for '{}'", shell_id))?;
    if !profile.exists() {
        return Ok((false, profile));
    }
    let source_line = source_hint(shell_id)
        .ok_or_else(|| format!("no source hint for '{}'", shell_id))?;

    let content = std::fs::read_to_string(&profile)
        .map_err(|e| format!("read profile: {}", e))?;

    if !content.lines().any(|l| l.trim() == source_line.trim()) {
        return Ok((false, profile));
    }

    let cleaned = strip_source_block(&content, &source_line);
    std::fs::write(&profile, cleaned)
        .map_err(|e| format!("write profile: {}", e))?;

    Ok((true, profile))
}

// ── Primary shell config ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DotfileConfig {
    pub primary_shell: Option<String>,
}

fn config_path() -> PathBuf {
    config_base_dir().join("config.json")
}

pub fn read_config() -> DotfileConfig {
    let path = config_path();
    if !path.exists() {
        return DotfileConfig::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn write_config(cfg: &DotfileConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create config dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("serialize config: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("write config: {}", e))
}

pub fn set_primary_shell(shell_id: &str) -> Result<(), String> {
    let mut cfg = read_config();
    cfg.primary_shell = Some(shell_id.to_string());
    write_config(&cfg)
}

pub fn clear_primary_shell() -> Result<(), String> {
    let mut cfg = read_config();
    cfg.primary_shell = None;
    write_config(&cfg)
}

/// Returns the effective primary shell: explicit config → system default → None.
pub fn effective_primary_shell() -> Option<String> {
    read_config().primary_shell.or_else(detect_default_shell)
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn script_content(id: &str) -> Option<&'static str> {
    match id {
        "bash"       => Some(BASH_SCRIPT),
        "zsh"        => Some(ZSH_SCRIPT),
        "fish"       => Some(FISH_SCRIPT),
        "nushell"    => Some(NU_SCRIPT),
        "powershell" => Some(POSH_SCRIPT),
        _ => None,
    }
}

fn has_script(id: &str) -> bool {
    matches!(id, "bash" | "zsh" | "fish" | "nushell" | "powershell")
}

fn strip_source_block(content: &str, source_line: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        if i + 2 < lines.len()
            && lines[i].trim().is_empty()
            && lines[i + 1].trim() == "# dotfile-rs"
            && lines[i + 2].trim() == source_line.trim()
        {
            i += 3;
            continue;
        }
        if lines[i].trim() == source_line.trim() {
            i += 1;
            continue;
        }
        out.push(lines[i]);
        i += 1;
    }
    let mut result = out.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn home_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    let key = "USERPROFILE";
    #[cfg(not(target_os = "windows"))]
    let key = "HOME";

    std::env::var(key)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}