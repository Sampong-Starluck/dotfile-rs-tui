package com.sampong.dotfile.service;

import com.sampong.dotfile.model.OperatingSystem;

public interface OsService {
    boolean isWindows();
    boolean isMac();
    boolean isLinux();

    /** "windows" | "macos" | "linux" — the key used in apps.json platforms maps */
    String osKey();

    OperatingSystem detect();
}
