package com.sampong.dotfile.model;

/** Port of Rust {@code models/os.rs::OperatingSystem}. Detection is Phase 3. */
public record OperatingSystem(Kind kind, LinuxDistro distro, String distroName) {
    public enum Kind { WINDOWS, MACOS, LINUX, UNKNOWN }

    public String label() {
        return switch (kind) {
            case WINDOWS -> "Windows"; case MACOS -> "macOS";
            case LINUX -> "Linux";     case UNKNOWN -> "Unknown";
        };
    }

    /** "Linux (Arch Linux)" — port of the Rust Display impl. */
    @Override public String toString() {
        if (kind == Kind.LINUX && distro != null) {
            String d = (distro == LinuxDistro.OTHER && distroName != null) ? distroName : distro.label();
            return "Linux (" + d + ")";
        }
        return label();
    }
}
