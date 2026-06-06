use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum OperatingSystem {
    Windows,
    MacOs,
    Linux { distro: Option<LinuxDistro> },
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinuxDistro {
    Debian,   // apt
    Fedora,   // dnf
    Arch,     // pacman / yay
    Void,     // xbps
    Other(String),
}

impl OperatingSystem {
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        return Self::Windows;

        #[cfg(target_os = "macos")]
        return Self::MacOs;

        #[cfg(target_os = "linux")]
        return Self::Linux {
            distro: LinuxDistro::detect(),
        };

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux"
        )))]
        return Self::Unknown;
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Windows          => "Windows",
            Self::MacOs            => "macOS",
            Self::Linux { .. }     => "Linux",
            Self::Unknown          => "Unknown",
        }
    }
}

impl LinuxDistro {
    /// Reads /etc/os-release to identify the distro
    pub fn detect() -> Option<Self> {
        let content = std::fs::read_to_string("/etc/os-release").ok()?;

        // Look for ID= or ID_LIKE= fields
        let id = content
            .lines()
            .find_map(|line| line.strip_prefix("ID="))
            .map(|s| s.trim_matches('"').to_lowercase());

        let id_like = content
            .lines()
            .find_map(|line| line.strip_prefix("ID_LIKE="))
            .map(|s| s.trim_matches('"').to_lowercase());

        // Check ID first, fall back to ID_LIKE for derivatives
        let combined = format!(
            "{} {}",
            id.as_deref().unwrap_or(""),
            id_like.as_deref().unwrap_or("")
        );

        if combined.contains("arch") {
            Some(Self::Arch)
        } else if combined.contains("fedora") || combined.contains("rhel") {
            Some(Self::Fedora)
        } else if combined.contains("debian") || combined.contains("ubuntu") {
            Some(Self::Debian)
        } else if combined.contains("void") {
            Some(Self::Void)
        } else {
            id.map(|name| Self::Other(name))
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Debian      => "Debian / Ubuntu",
            Self::Fedora      => "Fedora / RHEL",
            Self::Arch        => "Arch Linux",
            Self::Void        => "Void Linux",
            Self::Other(name) => name.as_str(),
        }
    }
}

impl fmt::Display for OperatingSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Linux { distro: Some(d) } => write!(f, "Linux ({})", d.label()),
            Self::Linux { distro: None }    => write!(f, "Linux"),
            other                           => write!(f, "{}", other.label()),
        }
    }
}