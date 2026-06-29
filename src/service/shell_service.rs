use std::path::{Path, PathBuf};
use nanoserde::DeJson;
use crate::models::{ShellEntry, ShellsFile};

const BEGIN_MARK: &str = "# >>> dotfile-rs profile >>>";
const END_MARK: &str = "# <<< dotfile-rs profile <<<";

pub fn read_shells_json() -> Result<Vec<ShellEntry>, Box<dyn std::error::Error>> {
    const SHELLS_JSON: &str = include_str!("../json/shells.json");
    let file: ShellsFile = DeJson::deserialize_json(SHELLS_JSON)
        .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;
    Ok(file.shells)
}

fn home_dir() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    } else {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

/// Directory where dotfile-rs stores the user's managed per-shell profile
/// scripts — Roaming AppData on Windows, XDG data dir on Linux/macOS.
pub fn profiles_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        let appdata = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir().unwrap_or_default().join("AppData").join("Roaming"));
        appdata.join("dotfile-rs").join("profiles")
    } else {
        let data_home = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir().unwrap_or_default().join(".local").join("share"));
        data_home.join("dotfile-rs").join("profiles")
    }
}

fn script_ext(id: &str) -> &'static str {
    match id {
        "powershell" => "ps1",
        "nushell"    => "nu",
        "bash"       => "sh",
        "fish"       => "fish",
        _            => "txt",
    }
}

/// Path to the user-editable managed script for a given shell id.
pub fn managed_script_path(id: &str) -> PathBuf {
    profiles_dir().join(format!("{}.{}", id, script_ext(id)))
}

/// Path to the shell's real rc/profile file that gets the source hook injected.
pub fn real_profile_path(id: &str) -> Option<PathBuf> {
    let home = home_dir()?;
    let config_home = || {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"))
    };

    match id {
        "powershell" => {
            if cfg!(target_os = "windows") {
                Some(home.join("Documents").join("PowerShell").join("Microsoft.PowerShell_profile.ps1"))
            } else {
                Some(config_home().join("powershell").join("Microsoft.PowerShell_profile.ps1"))
            }
        }
        "nushell" => {
            if cfg!(target_os = "windows") {
                let appdata = std::env::var_os("APPDATA")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| home.join("AppData").join("Roaming"));
                Some(appdata.join("nushell").join("config.nu"))
            } else {
                Some(config_home().join("nushell").join("config.nu"))
            }
        }
        "bash" => Some(home.join(".bashrc")),
        "fish" => Some(config_home().join("fish").join("config.fish")),
        _ => None,
    }
}

fn shell_binaries(id: &str) -> &'static [&'static str] {
    match id {
        "powershell" => &["pwsh", "powershell"],
        "nushell"    => &["nu"],
        "bash"       => &["bash"],
        "fish"       => &["fish"],
        _            => &[],
    }
}

/// Best-effort detection of whether the shell binary is on PATH.
pub fn is_shell_installed(id: &str) -> bool {
    shell_binaries(id).iter().any(|bin| which::which(bin).is_ok())
}

/// The exact line that gets injected into the real profile to load the
/// managed script — shown in the UI so the user knows what "apply" does.
pub fn source_line(id: &str, path: &Path) -> String {
    let p = path.display();
    match id {
        "powershell" => format!(". \"{}\"", p),
        _            => format!("source \"{}\"", p),
    }
}

pub fn default_template(name: &str) -> String {
    format!(
        "# dotfile-rs managed {} profile\n# Add your personal aliases, functions and config below.\n",
        name
    )
}

/// True if the shell's real profile already sources the managed script.
pub fn is_shell_applied(id: &str) -> bool {
    let Some(path) = real_profile_path(id) else { return false };
    std::fs::read_to_string(path)
        .map(|content| content.contains(BEGIN_MARK))
        .unwrap_or(false)
}

/// Returns profile content with the dotfile-rs source block appended, or
/// `None` if the block is already present (caller should treat as a no-op).
fn insert_block(existing: &str, id: &str, managed: &Path) -> Option<String> {
    if existing.contains(BEGIN_MARK) {
        return None;
    }
    let separator = if existing.is_empty() || existing.ends_with('\n') { "" } else { "\n" };
    let block = format!("{}{}\n{}\n{}\n", separator, BEGIN_MARK, source_line(id, managed), END_MARK);
    let mut new_content = existing.to_string();
    new_content.push_str(&block);
    Some(new_content)
}

/// Returns profile content with the dotfile-rs source block stripped out.
fn remove_block(existing: &str) -> String {
    let mut out = String::new();
    let mut in_block = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed == BEGIN_MARK {
            in_block = true;
            continue;
        }
        if trimmed == END_MARK {
            in_block = false;
            continue;
        }
        if in_block {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Ensures the managed script exists and the real shell profile sources it.
/// Idempotent — calling twice has no extra effect.
pub fn apply_shell(id: &str, name: &str) -> Result<(), String> {
    let managed = managed_script_path(id);
    if let Some(parent) = managed.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if !managed.exists() {
        std::fs::write(&managed, default_template(name)).map_err(|e| e.to_string())?;
    }

    let real = real_profile_path(id).ok_or_else(|| format!("no known profile path for {}", id))?;
    if let Some(parent) = real.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let existing = std::fs::read_to_string(&real).unwrap_or_default();
    match insert_block(&existing, id, &managed) {
        Some(new_content) => std::fs::write(&real, new_content).map_err(|e| e.to_string()),
        None => Ok(()), // already applied
    }
}

/// Removes the dotfile-rs source block from the real shell profile, if present.
/// Leaves the managed script itself untouched.
pub fn unapply_shell(id: &str) -> Result<(), String> {
    let Some(real) = real_profile_path(id) else { return Ok(()) };
    let Ok(existing) = std::fs::read_to_string(&real) else { return Ok(()) };
    if !existing.contains(BEGIN_MARK) {
        return Ok(());
    }
    std::fs::write(&real, remove_block(&existing)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_block_appends_marker_and_source_line() {
        let managed = PathBuf::from("/tmp/dotfile-rs/profiles/bash.sh");
        let result = insert_block("existing line\n", "bash", &managed).unwrap();
        assert!(result.starts_with("existing line\n"));
        assert!(result.contains(BEGIN_MARK));
        assert!(result.contains(&source_line("bash", &managed)));
        assert!(result.contains(END_MARK));
    }

    #[test]
    fn insert_block_is_idempotent() {
        let managed = PathBuf::from("/tmp/dotfile-rs/profiles/bash.sh");
        let once = insert_block("", "bash", &managed).unwrap();
        assert!(insert_block(&once, "bash", &managed).is_none());
    }

    #[test]
    fn insert_block_adds_newline_separator_when_missing() {
        let managed = PathBuf::from("/tmp/dotfile-rs/profiles/bash.sh");
        let result = insert_block("no trailing newline", "bash", &managed).unwrap();
        assert!(result.starts_with("no trailing newline\n# >>>"));
    }

    #[test]
    fn remove_block_strips_only_the_managed_section() {
        let managed = PathBuf::from("/tmp/dotfile-rs/profiles/bash.sh");
        let applied = insert_block("alias ll='ls -la'\n", "bash", &managed).unwrap();
        let removed = remove_block(&applied);
        assert_eq!(removed, "alias ll='ls -la'\n");
        assert!(!removed.contains(BEGIN_MARK));
    }

    #[test]
    fn remove_block_on_unapplied_content_is_unchanged() {
        let content = "alias ll='ls -la'\n";
        assert_eq!(remove_block(content), content);
    }

    #[test]
    fn powershell_uses_dot_source_others_use_source_keyword() {
        let path = PathBuf::from("/tmp/profile.ps1");
        assert!(source_line("powershell", &path).starts_with(". \""));
        assert!(source_line("bash", &path).starts_with("source \""));
        assert!(source_line("fish", &path).starts_with("source \""));
        assert!(source_line("nushell", &path).starts_with("source \""));
    }
}