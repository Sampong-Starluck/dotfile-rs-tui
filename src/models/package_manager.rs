// models/package_manager.rs
use crate::models::os::{LinuxDistro, OperatingSystem};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Winget,
    Scoop,   // ← add
    Choco,   // ← add
    Apt,
    Dnf,
    Pacman,
    Yay,
    Xbps,
    Brew,   // ← add this
}

pub struct PmCommand {
    pub name:        &'static str,
    pub command:     &'static str,
    pub description: &'static str,
}


impl PackageManager {
    pub fn commands(&self) -> &'static [PmCommand] {
        match self {
            Self::Apt => &[
                PmCommand { name: "install",  command: "apt install <pkg>",     description: "Install a package"          },
                PmCommand { name: "remove",   command: "apt remove <pkg>",      description: "Remove a package"           },
                PmCommand { name: "update",   command: "apt update",            description: "Refresh package index"      },
                PmCommand { name: "upgrade",  command: "apt upgrade",           description: "Upgrade all packages"       },
                PmCommand { name: "search",   command: "apt search <pkg>",      description: "Search available packages"  },
                PmCommand { name: "show",     command: "apt show <pkg>",        description: "Show package details"       },
                PmCommand { name: "list",     command: "apt list --installed",  description: "List installed packages"    },
            ],
            Self::Dnf => &[
                PmCommand { name: "install",  command: "dnf install <pkg>",     description: "Install a package"          },
                PmCommand { name: "remove",   command: "dnf remove <pkg>",      description: "Remove a package"           },
                PmCommand { name: "update",   command: "dnf update",            description: "Update all packages"        },
                PmCommand { name: "search",   command: "dnf search <pkg>",      description: "Search available packages"  },
                PmCommand { name: "info",     command: "dnf info <pkg>",        description: "Show package details"       },
                PmCommand { name: "list",     command: "dnf list installed",    description: "List installed packages"    },
            ],
            Self::Pacman => &[
                PmCommand { name: "install",  command: "pacman -S <pkg>",       description: "Install a package"          },
                PmCommand { name: "remove",   command: "pacman -R <pkg>",       description: "Remove a package"           },
                PmCommand { name: "update",   command: "pacman -Syu",           description: "Sync and upgrade all"       },
                PmCommand { name: "search",   command: "pacman -Ss <pkg>",      description: "Search available packages"  },
                PmCommand { name: "info",     command: "pacman -Si <pkg>",      description: "Show package details"       },
                PmCommand { name: "list",     command: "pacman -Q",             description: "List installed packages"    },
            ],
            Self::Yay => &[
                PmCommand { name: "install",  command: "yay -S <pkg>",          description: "Install from AUR or repos"  },
                PmCommand { name: "remove",   command: "yay -R <pkg>",          description: "Remove a package"           },
                PmCommand { name: "update",   command: "yay -Syu",              description: "Upgrade all including AUR"  },
                PmCommand { name: "search",   command: "yay -Ss <pkg>",         description: "Search AUR and repos"       },
                PmCommand { name: "info",     command: "yay -Si <pkg>",         description: "Show package details"       },
                PmCommand { name: "list",     command: "yay -Q",                description: "List installed packages"    },
            ],
            Self::Xbps => &[
                PmCommand { name: "install",  command: "xbps-install <pkg>",    description: "Install a package"          },
                PmCommand { name: "remove",   command: "xbps-remove <pkg>",     description: "Remove a package"           },
                PmCommand { name: "update",   command: "xbps-install -Su",      description: "Sync and upgrade all"       },
                PmCommand { name: "search",   command: "xbps-query -Rs <pkg>",  description: "Search available packages"  },
                PmCommand { name: "info",     command: "xbps-query -S <pkg>",   description: "Show package details"       },
                PmCommand { name: "list",     command: "xbps-query -l",         description: "List installed packages"    },
            ],
            Self::Winget => &[
                PmCommand { name: "install",  command: "winget install <pkg>",  description: "Install a package"          },
                PmCommand { name: "remove",   command: "winget uninstall <pkg>",description: "Remove a package"           },
                PmCommand { name: "update",   command: "winget upgrade --all",  description: "Upgrade all packages"       },
                PmCommand { name: "search",   command: "winget search <pkg>",   description: "Search available packages"  },
                PmCommand { name: "show",     command: "winget show <pkg>",     description: "Show package details"       },
                PmCommand { name: "list",     command: "winget list",           description: "List installed packages"    },
            ],
            Self::Scoop => &[
                PmCommand { name: "install", command: "scoop install <pkg>",   description: "Install a package"         },
                PmCommand { name: "remove",  command: "scoop uninstall <pkg>", description: "Remove a package"          },
                PmCommand { name: "update",  command: "scoop update *",        description: "Upgrade all packages"      },
                PmCommand { name: "search",  command: "scoop search <pkg>",    description: "Search available packages" },
                PmCommand { name: "info",    command: "scoop info <pkg>",      description: "Show package details"      },
                PmCommand { name: "list",    command: "scoop list",            description: "List installed packages"   },
            ],
            Self::Choco => &[
                PmCommand { name: "install", command: "choco install <pkg> -y",   description: "Install a package"         },
                PmCommand { name: "remove",  command: "choco uninstall <pkg> -y", description: "Remove a package"          },
                PmCommand { name: "update",  command: "choco upgrade all -y",     description: "Upgrade all packages"      },
                PmCommand { name: "search",  command: "choco search <pkg>",       description: "Search available packages" },
                PmCommand { name: "info",    command: "choco info <pkg>",         description: "Show package details"      },
                PmCommand { name: "list",    command: "choco list --local-only",  description: "List installed packages"   },
            ],
            Self::Brew => &[
                PmCommand { name: "install",  command: "brew install <pkg>",    description: "Install a package"          },
                PmCommand { name: "remove",   command: "brew uninstall <pkg>",  description: "Remove a package"           },
                PmCommand { name: "update",   command: "brew update",           description: "Refresh Homebrew itself"    },
                PmCommand { name: "upgrade",  command: "brew upgrade",          description: "Upgrade all packages"       },
                PmCommand { name: "search",   command: "brew search <pkg>",     description: "Search available packages"  },
                PmCommand { name: "info",     command: "brew info <pkg>",       description: "Show package details"       },
                PmCommand { name: "list",     command: "brew list",             description: "List installed packages"    },
            ],
        }
    }
    /// Returns the binary name used to detect presence on PATH
    pub fn binary(&self) -> &'static str {
        match self {
            Self::Winget => "winget",
            Self::Scoop  => "scoop",   // ← add
            Self::Choco  => "choco",   // ← add
            Self::Apt    => "apt",
            Self::Dnf    => "dnf",
            Self::Pacman => "pacman",
            Self::Yay    => "yay",
            Self::Xbps   => "xbps-install",
            Self::Brew   => "brew",       // ← add this
        }
    }

    /// Human-readable label for the TUI
    pub fn label(&self) -> &'static str {
        match self {
            Self::Winget => "winget",
            Self::Scoop  => "scoop",   // ← add
            Self::Choco  => "choco",   // ← add
            Self::Apt    => "apt",
            Self::Dnf    => "dnf",
            Self::Pacman => "pacman",
            Self::Yay    => "yay (AUR)",
            Self::Xbps   => "xbps",
            Self::Brew   => "Homebrew",   // ← add this
        }
    }

    // All known package managers to probe
    // pub fn all() -> &'static [PackageManager] {
    //     &[
    //         Self::Winget,
    //         Self::Apt,
    //         Self::Dnf,
    //         Self::Pacman,
    //         Self::Yay,
    //         Self::Xbps,
    //         Self::Brew,   // ← add this
    //     ]
    // }

    /// Check if this package manager is available on the current system
    pub fn is_available(&self) -> bool {
        which::which(self.binary()).is_ok()
    }

    /// Detect all package managers present on this system
    pub fn detect() -> Vec<PackageManager> {
        let os = OperatingSystem::detect();

        // Filter candidates by OS first, then check PATH
        Self::candidates_for(&os)
            .into_iter()
            .filter(|pm| pm.is_available())
            .collect()
    }

    fn candidates_for(os: &OperatingSystem) -> Vec<PackageManager> {
        match os {
            OperatingSystem::Windows => vec![Self::Winget, Self::Scoop, Self::Choco],
            OperatingSystem::MacOs   => vec![Self::Brew],
            OperatingSystem::Linux { distro } => match distro {
                Some(LinuxDistro::Arch)   => vec![Self::Pacman, Self::Yay],
                Some(LinuxDistro::Debian) => vec![Self::Apt],
                Some(LinuxDistro::Fedora) => vec![Self::Dnf],
                Some(LinuxDistro::Void)   => vec![Self::Xbps],
                // Unknown distro — probe everything
                _ => vec![Self::Apt, Self::Dnf, Self::Pacman, Self::Yay, Self::Xbps],
            },
            _ => vec![],
        }
    }
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}