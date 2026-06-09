pub fn is_root() -> bool {
    #[cfg(unix)]
    { unsafe { libc::getuid() == 0 } }
    #[cfg(windows)]
    { false }
}

pub fn requires_sudo(mgr: &str) -> bool {
    matches!(mgr, "pacman" | "apt" | "apt-get" | "dnf" | "yum" | "xbps-install" | "apk")
}

pub fn requires_interactive(mgr: &str) -> bool {
    matches!(mgr,
        "pacman" | "apt" | "apt-get" | "dnf" | "yum" | "xbps-install" | "apk" |
        "yay" | "paru" | "choco"
    )
}

