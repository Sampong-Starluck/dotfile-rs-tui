package com.sampong.dotfile.service;

import com.sampong.dotfile.model.SearchResult;

import java.util.List;
import java.util.concurrent.CompletableFuture;

/**
 * Async wrapper over live package search / installed-list queries, polled per frame by the
 * feature controllers (PLAN.md §6). Phase 7 wires the futures/drains with a stub body;
 * Phase 9 replaces the body with real process spawning + output parsing
 * ({@link SearchService} + {@link OutputParsers}).
 */
public interface PackageQueryService {
    CompletableFuture<List<SearchResult>> search(String mgr, String query);

    CompletableFuture<List<SearchResult>> listInstalled(String mgr);
}
