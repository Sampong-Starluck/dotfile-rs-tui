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
}
