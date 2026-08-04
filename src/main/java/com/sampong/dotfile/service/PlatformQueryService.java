package com.sampong.dotfile.service;

import com.sampong.dotfile.model.PackageManager;

import java.util.List;
import java.util.concurrent.CompletableFuture;

/**
 * Async wrapper over {@link PackageManagerService#detect()} (N x PATH probes) so the
 * Managers panel never blocks the startup path (PLAN.md 5b).
 */
public interface PlatformQueryService {
    CompletableFuture<List<PackageManager>> detectManagers();
}
