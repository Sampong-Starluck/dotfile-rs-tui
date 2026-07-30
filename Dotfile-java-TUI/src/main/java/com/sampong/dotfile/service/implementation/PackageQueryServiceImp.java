package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.model.SearchResult;
import com.sampong.dotfile.service.PackageQueryService;
import lombok.extern.slf4j.Slf4j;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;

import java.util.List;
import java.util.concurrent.CompletableFuture;

/**
 * Phase 7 stub: wires the async future/drain plumbing (PLAN.md §6) without spawning a real
 * process yet. {@code search} returns one fake row per PLAN.md phase-07 §7.4 so the search flow
 * is visually verifiable; {@code listInstalled} returns an empty list. Phase 9 replaces both
 * bodies with real {@code ProcessBuilder} spawning + {@code OutputParsers} decoding.
 */
@Slf4j
@Service
public class PackageQueryServiceImp implements PackageQueryService {

    @Override
    @Async
    public CompletableFuture<List<SearchResult>> search(String mgr, String query) {
        log.debug("search stub: mgr={} query={}", mgr, query);
        SearchResult fake = new SearchResult(query, query, "0.0.0");
        return CompletableFuture.completedFuture(List.of(fake));
    }

    @Override
    @Async
    public CompletableFuture<List<SearchResult>> listInstalled(String mgr) {
        log.debug("listInstalled stub: mgr={}", mgr);
        return CompletableFuture.completedFuture(List.of());
    }
}
