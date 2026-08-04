package com.sampong.dotfile.service;

import com.sampong.dotfile.model.AppEntry;

import java.util.Optional;

public interface InstallCommandService {
    /**
     * Port of {@code get_install_command()}: resolve entry's package id for this manager,
     * falling back to the OS's preferred manager order when detectedMgr is absent.
     */
    Optional<String> installCommandFor(AppEntry entry, String detectedMgr);

    String installCommand(String mgr, String pkg);

    String removeCommand(String mgr, String pkg);

    /** Upgrade a single already-installed package to its available version (Phase 13,
     *  net-new — no Rust prior art; the original app only has whole-system upgrade commands,
     *  already ported as {@code PmCommand}). */
    String updateCommand(String mgr, String pkg);
}
