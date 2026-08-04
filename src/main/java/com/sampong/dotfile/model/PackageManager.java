package com.sampong.dotfile.model;

import java.util.List;

/**
 * Data only — port of Rust {@code models/package_manager.rs}. Detection is Phase 3.
 * Command tables and descriptions ported verbatim from the Rust source.
 */
public enum PackageManager {
    WINGET("winget", "winget"),   SCOOP("scoop", "scoop"),   CHOCO("choco", "choco"),
    APT("apt", "apt"),            DNF("dnf", "dnf"),         PACMAN("pacman", "pacman"),
    YAY("yay", "yay (AUR)"),      PARU("paru", "paru (AUR)"),
    XBPS("xbps-install", "xbps"), BREW("brew", "Homebrew");

    private final String binary;
    private final String label;
    PackageManager(String binary, String label) { this.binary = binary; this.label = label; }
    public String binary() { return binary; }
    public String label()  { return label; }

    public List<PmCommand> commands() {
        return switch (this) {
            case APT -> List.of(
                new PmCommand("install", "apt install <pkg>",    "Install a package"),
                new PmCommand("remove",  "apt remove <pkg>",     "Remove a package"),
                new PmCommand("update",  "apt update",           "Refresh package index"),
                new PmCommand("upgrade", "apt upgrade",          "Upgrade all packages"),
                new PmCommand("search",  "apt search <pkg>",     "Search available packages"),
                new PmCommand("show",    "apt show <pkg>",       "Show package details"),
                new PmCommand("list",    "apt list --installed", "List installed packages"));
            case DNF -> List.of(
                new PmCommand("install", "dnf install <pkg>",  "Install a package"),
                new PmCommand("remove",  "dnf remove <pkg>",   "Remove a package"),
                new PmCommand("update",  "dnf update",         "Update all packages"),
                new PmCommand("search",  "dnf search <pkg>",   "Search available packages"),
                new PmCommand("info",    "dnf info <pkg>",     "Show package details"),
                new PmCommand("list",    "dnf list installed", "List installed packages"));
            case PACMAN -> List.of(
                new PmCommand("install", "pacman -S <pkg>",  "Install a package"),
                new PmCommand("remove",  "pacman -R <pkg>",  "Remove a package"),
                new PmCommand("update",  "pacman -Syu",      "Sync and upgrade all"),
                new PmCommand("search",  "pacman -Ss <pkg>", "Search available packages"),
                new PmCommand("info",    "pacman -Si <pkg>", "Show package details"),
                new PmCommand("list",    "pacman -Q",        "List installed packages"));
            case YAY -> List.of(
                new PmCommand("install", "yay -S <pkg>",  "Install from AUR or repos"),
                new PmCommand("remove",  "yay -R <pkg>",  "Remove a package"),
                new PmCommand("update",  "yay -Syu",      "Upgrade all including AUR"),
                new PmCommand("search",  "yay -Ss <pkg>", "Search AUR and repos"),
                new PmCommand("info",    "yay -Si <pkg>", "Show package details"),
                new PmCommand("list",    "yay -Q",        "List installed packages"));
            case PARU -> List.of(
                new PmCommand("install", "paru -S <pkg>",  "Install from AUR or repos"),
                new PmCommand("remove",  "paru -R <pkg>",  "Remove a package"),
                new PmCommand("update",  "paru -Syu",      "Upgrade all including AUR"),
                new PmCommand("search",  "paru -Ss <pkg>", "Search AUR and repos"),
                new PmCommand("info",    "paru -Si <pkg>", "Show package details"),
                new PmCommand("list",    "paru -Q",        "List installed packages"));
            case XBPS -> List.of(
                new PmCommand("install", "xbps-install <pkg>",   "Install a package"),
                new PmCommand("remove",  "xbps-remove <pkg>",    "Remove a package"),
                new PmCommand("update",  "xbps-install -Su",     "Sync and upgrade all"),
                new PmCommand("search",  "xbps-query -Rs <pkg>", "Search available packages"),
                new PmCommand("info",    "xbps-query -S <pkg>",  "Show package details"),
                new PmCommand("list",    "xbps-query -l",        "List installed packages"));
            case WINGET -> List.of(
                new PmCommand("install", "winget install <pkg>",   "Install a package"),
                new PmCommand("remove",  "winget uninstall <pkg>", "Remove a package"),
                new PmCommand("update",  "winget upgrade --all",   "Upgrade all packages"),
                new PmCommand("search",  "winget search <pkg>",    "Search available packages"),
                new PmCommand("show",    "winget show <pkg>",      "Show package details"),
                new PmCommand("list",    "winget list",            "List installed packages"));
            case SCOOP -> List.of(
                new PmCommand("install", "scoop install <pkg>",   "Install a package"),
                new PmCommand("remove",  "scoop uninstall <pkg>", "Remove a package"),
                new PmCommand("update",  "scoop update *",        "Upgrade all packages"),
                new PmCommand("search",  "scoop search <pkg>",    "Search available packages"),
                new PmCommand("info",    "scoop info <pkg>",      "Show package details"),
                new PmCommand("list",    "scoop list",            "List installed packages"));
            case CHOCO -> List.of(
                new PmCommand("install", "choco install <pkg> -y",   "Install a package"),
                new PmCommand("remove",  "choco uninstall <pkg> -y", "Remove a package"),
                new PmCommand("update",  "choco upgrade all -y",     "Upgrade all packages"),
                new PmCommand("search",  "choco search <pkg>",       "Search available packages"),
                new PmCommand("info",    "choco info <pkg>",         "Show package details"),
                new PmCommand("list",    "choco list --local-only",  "List installed packages"));
            case BREW -> List.of(
                new PmCommand("install", "brew install <pkg>",   "Install a package"),
                new PmCommand("remove",  "brew uninstall <pkg>", "Remove a package"),
                new PmCommand("update",  "brew update",          "Refresh Homebrew itself"),
                new PmCommand("upgrade", "brew upgrade",         "Upgrade all packages"),
                new PmCommand("search",  "brew search <pkg>",    "Search available packages"),
                new PmCommand("info",    "brew info <pkg>",      "Show package details"),
                new PmCommand("list",    "brew list",            "List installed packages"));
        };
    }

    public String description() {
        return switch (this) {
            case WINGET -> "Windows built-in, largest catalog";
            case SCOOP  -> "portable installs, no admin needed";
            case CHOCO  -> "traditional installs, wide support";
            case APT    -> "Debian/Ubuntu";
            case DNF    -> "Red Hat/Fedora";
            case PACMAN -> "Arch Linux default package manager";
            case YAY    -> "Yay (Yet Another Yogurt) AUR helper";
            case PARU   -> "Paru — feature-rich AUR helper";
            case XBPS   -> "Void Linux";
            case BREW   -> "";
        };
    }

    @Override public String toString() { return label; }
}
