package com.sampong.dotfile.service;

import com.sampong.dotfile.model.SearchResult;

import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

/**
 * Async wrapper over live package search / installed-list queries, polled per frame by the
 * feature controllers (PLAN.md §6). Spawns the manager CLI and parses its output via
 * {@link SearchService} + {@link OutputParsers}.
 */
public interface PackageQueryService {
    CompletableFuture<List<SearchResult>> search(String mgr, String query);

    CompletableFuture<List<SearchResult>> listInstalled(String mgr);

    /** id -> available-version for packages with a pending update (Phase 13, net-new). Resolves
     *  to an empty map, without spawning any process, for managers with no known update-check
     *  command yet ({@link SearchService#upgradeListCommand} empty). */
    CompletableFuture<Map<String, String>> checkUpdates(String mgr);
}
