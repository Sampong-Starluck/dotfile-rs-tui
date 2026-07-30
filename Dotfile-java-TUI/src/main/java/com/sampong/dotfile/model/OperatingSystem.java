package com.sampong.dotfile.model;

import org.jspecify.annotations.Nullable;

/** Port of Rust {@code models/os.rs::OperatingSystem}. Detection is Phase 3. */
public record OperatingSystem(Kind kind, @Nullable LinuxDistro distro, @Nullable String distroName) {
    public enum Kind { WINDOWS, MACOS, LINUX, UNKNOWN }

    public String label() {
        return switch (kind) {
            case WINDOWS -> "Windows"; case MACOS -> "macOS";
            case LINUX -> "Linux";     case UNKNOWN -> "Unknown";
        };
    }

    /** "windows" | "macos" | "linux" — the key used in apps.json platforms maps (matches {@code OsService.osKey()}). */
    public String key() {
        return switch (kind) {
            case WINDOWS -> "windows";
            case MACOS -> "macos";
            case LINUX, UNKNOWN -> "linux";
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
