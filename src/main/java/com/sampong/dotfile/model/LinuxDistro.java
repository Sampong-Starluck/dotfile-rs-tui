package com.sampong.dotfile.model;

/** Port of Rust {@code models/os.rs::LinuxDistro}. */
public enum LinuxDistro {
    DEBIAN("Debian / Ubuntu"), FEDORA("Fedora / RHEL"), ARCH("Arch Linux"),
    VOID("Void Linux"), OTHER("Other");

    private final String label;
    LinuxDistro(String label) { this.label = label; }
    public String label() { return label; }
}
