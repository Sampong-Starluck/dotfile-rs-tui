package com.sampong.dotfile.service;

import com.sampong.dotfile.model.OperatingSystem;
import com.sampong.dotfile.model.PackageManager;

import java.util.List;

public interface PackageManagerService {
    /** Port of {@code PackageManager::detect()} — OS-filtered candidates, then PATH check. */
    List<PackageManager> detect();

    /** Port of {@code candidates_for()} — copy the exact mapping from package_manager.rs:174. */
    List<PackageManager> candidatesFor(OperatingSystem os);
}
