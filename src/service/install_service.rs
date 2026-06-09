use crate::models::AppEntry;

pub fn get_install_command(entry: &AppEntry, detected_mgr: &str) -> Option<String> {
    let os = if cfg!(target_os = "windows") { "windows" }
    else if cfg!(target_os = "macos") { "macos" }
    else { "linux" };

    let platform = entry.platforms.get(os)?;
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
    Some(install_command(mgr, pkg))
}

pub fn install_command(mgr: &str, pkg: &str) -> String {
    match mgr {
        "pacman"       => format!("pacman -S {}", pkg),
        "yay"          => format!("yay -S {}", pkg),
        "paru"         => format!("paru -S {}", pkg),
        "apt"          => format!("apt install -y {}", pkg),
        "apt-get"      => format!("apt-get install -y {}", pkg),
        "dnf"          => format!("dnf install -y {}", pkg),
        "yum"          => format!("yum install -y {}", pkg),
        "xbps-install" => format!("xbps-install -S {}", pkg),
        "apk"          => format!("apk add {}", pkg),
        "brew"         => format!("brew install {}", pkg),
        "winget"       => format!("winget install --id {} -e", pkg),
        "scoop"        => format!("scoop install {}", pkg),
        "choco"        => format!("choco install {} -y", pkg),
        _              => format!("{} install {}", mgr, pkg),
    }
}