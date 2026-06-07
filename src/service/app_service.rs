use crate::models::{AppEntry, AppSection, Apps};

pub fn read_apps_json() -> Result<Apps, Box<dyn std::error::Error>> {
    const APPS_JSON: &str = include_str!("../json/apps.json");
    println!("JSON loaded, length: {}", APPS_JSON.len()); // ← confirm it loaded
    let apps: Apps = serde_json::from_str(APPS_JSON)?;
    Ok(apps)
}

pub fn filter_apps_by_platform(apps: &Apps, detected_mgr: &str) -> Apps {
    let os = if cfg!(target_os = "windows") { "windows" }
    else if cfg!(target_os = "macos") { "macos" }
    else { "linux" };

    apps.iter()
        .filter_map(|section| {
            let filtered_apps: Vec<AppEntry> = section.apps
                .iter()
                .filter(|entry| {
                    // Must support current OS
                    let Some(platform) = entry.platforms.get(os) else {
                        return false;
                    };
                    // Must support detected package manager
                    platform.contains_key(detected_mgr)
                })
                .cloned()
                .collect();

            if filtered_apps.is_empty() {
                None
            } else {
                Some(AppSection {
                    section: section.section.clone(),
                    apps: filtered_apps,
                })
            }
        })
        .collect()
}

// pub fn get_install_command(app: &AppEntry) -> Option<String> {
//     let os = if cfg!(target_os = "windows") { "windows" }
//     else if cfg!(target_os = "macos") { "macos" }
//     else { "linux" };
//
//     let platform = app.platforms.get(os)?;
//
//     let managers: &[&str] = match os {
//         "windows" => &["winget", "scoop", "choco"],
//         "macos"   => &["brew"],
//         _         => &["pacman", "apt", "dnf", "xbps"],
//     };
//
//     for mgr in managers {
//         if let Some(pkg) = platform.get(*mgr) {
//             return Some(format!("{} install {}", mgr, pkg));
//         }
//     }
//     None
// }

pub fn is_root() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::getuid() == 0 }
    }
    #[cfg(windows)]
    {
        false // windows uses UAC, not root
    }
}

pub fn requires_sudo(mgr: &str) -> bool {
    matches!(mgr, "pacman" | "apt" | "apt-get" | "dnf" | "yum" | "xbps-install" | "apk")
}

pub fn get_install_command(entry: &AppEntry, detected_mgr: &str) -> Option<String> {
    let os = if cfg!(target_os = "windows") { "windows" }
    else if cfg!(target_os = "macos") { "macos" }
    else { "linux" };

    let platform = entry.platforms.get(os)?;

    // Try detected manager first, then fallback
    let mgr = if platform.contains_key(detected_mgr) {
        detected_mgr
    } else {
        let fallbacks: &[&str] = match os {
            "windows" => &["winget", "scoop", "choco"],
            "macos"   => &["brew"],
            _         => &["pacman", "yay", "apt", "dnf", "xbps-install"],
        };
        fallbacks.iter().find(|m| platform.contains_key(**m)).copied()?
    };

    let pkg = platform.get(mgr)?;
    let cmd = install_command(mgr, pkg);
    Some(cmd)
}

pub fn install_command(mgr: &str, pkg: &str) -> String {
    match mgr {
        // Arch
        "pacman"       => format!("pacman -S {}", pkg),
        "yay"          => format!("yay -S {}", pkg),
        "paru"         => format!("paru -S {}", pkg),
        // Debian/Ubuntu
        "apt"          => format!("apt install -y {}", pkg),
        "apt-get"      => format!("apt-get install -y {}", pkg),
        // Fedora/RHEL
        "dnf"          => format!("dnf install -y {}", pkg),
        "yum"          => format!("yum install -y {}", pkg),
        // Void
        "xbps-install" => format!("xbps-install -S {}", pkg),
        // Alpine
        "apk"          => format!("apk add {}", pkg),
        // macOS
        "brew"         => format!("brew install {}", pkg),
        // Windows
        "winget"       => format!("winget install --id {} -e", pkg),
        "scoop"        => format!("scoop install {}", pkg),
        "choco"        => format!("choco install {} -y", pkg),
        // fallback
        _              => format!("{} install {}", mgr, pkg),
    }
}
pub fn requires_interactive(mgr: &str) -> bool {
    matches!(mgr,
        "pacman" | "apt" | "apt-get" | "dnf" | "yum" | "xbps-install" | "apk" |
        "yay" | "paru"
    )
}