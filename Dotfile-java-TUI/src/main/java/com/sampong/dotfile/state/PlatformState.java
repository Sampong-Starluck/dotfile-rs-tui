package com.sampong.dotfile.state;

import com.sampong.dotfile.model.OperatingSystem;
import com.sampong.dotfile.model.PackageManager;

import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/** Managers panel + active manager. Lazily detected per the 200 ms rule (PLAN.md 5b). */
public class PlatformState {
    public OperatingSystem os;
    public List<PackageManager> packageManagers = List.of();
    public boolean detecting = true;
    public CompletableFuture<List<PackageManager>> detectFuture = null;
    public int selectedPm = 0;
    public int managersCursor = 0;
    public int commandScroll = 0;

    public String activeBinary() {
        return selectedManager().map(PackageManager::binary).orElse("unknown");
    }

    public Optional<PackageManager> selectedManager() {
        if (packageManagers.isEmpty() || selectedPm < 0 || selectedPm >= packageManagers.size()) {
            return Optional.empty();
        }
        return Optional.of(packageManagers.get(selectedPm));
    }

    public void moveCursor(int delta) {
        if (packageManagers.isEmpty()) {
            managersCursor = 0;
            return;
        }
        managersCursor = Math.floorMod(managersCursor + delta, packageManagers.size());
    }

    /** Package-private: only {@link AppState#activateManager(int)} may cross-reset other features. */
    boolean activate(int idx) {
        if (idx < 0 || idx >= packageManagers.size() || idx == selectedPm) {
            return false;
        }
        selectedPm = idx;
        return true;
    }
}
