package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.model.SearchResult;
import com.sampong.dotfile.service.OutputParsers;
import com.sampong.dotfile.service.PackageQueryService;
import com.sampong.dotfile.service.SearchService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

/**
 * Port of {@code app_tab.rs::run_search}/{@code run_list_installed} (PLAN.md phase-09 §9.1):
 * spawns the manager CLI, decodes its output (winget's UTF-16/BOM quirks via
 * {@link com.sampong.dotfile.service.DecodeUtil}), and parses it with {@link OutputParsers}.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class PackageQueryServiceImp implements PackageQueryService {

    private final SearchService searchService;

    @Override
    @Async
    public CompletableFuture<List<SearchResult>> search(String mgr, String query) {
        try {
            SearchService.Cmd cmd = searchService.searchCommand(mgr, query);
            byte[] out = runAndCapture(cmd);
            String text = OutputParsers.decodeSearchOutput(mgr, out);
            List<SearchResult> results = OutputParsers.parseSearchOutput(mgr, text);
            log.debug("search: {} result(s) for '{}' via {}", results.size(), query, mgr);
            return CompletableFuture.completedFuture(results);
        } catch (Exception e) {
            log.error("search via {} failed", mgr, e);
            return CompletableFuture.completedFuture(List.of());
        }
    }

    @Override
    @Async
    public CompletableFuture<List<SearchResult>> listInstalled(String mgr) {
        try {
            SearchService.Cmd cmd = searchService.listCommand(mgr);
            byte[] out = runAndCapture(cmd);
            String text = OutputParsers.decodeSearchOutput(mgr, out);
            List<SearchResult> results = OutputParsers.parseListOutput(mgr, text);
            log.debug("listInstalled: {} package(s) via {}", results.size(), mgr);
            return CompletableFuture.completedFuture(results);
        } catch (Exception e) {
            log.error("listInstalled via {} failed", mgr, e);
            return CompletableFuture.completedFuture(List.of());
        }
    }

    @Override
    @Async
    public CompletableFuture<Map<String, String>> checkUpdates(String mgr) {
        var cmdOpt = searchService.upgradeListCommand(mgr);
        if (cmdOpt.isEmpty()) {
            return CompletableFuture.completedFuture(Map.of());
        }
        try {
            byte[] out = runAndCapture(cmdOpt.get());
            String text = OutputParsers.decodeSearchOutput(mgr, out);
            Map<String, String> updates = OutputParsers.parseUpgradeOutput(mgr, text);
            log.debug("checkUpdates: {} update(s) available via {}", updates.size(), mgr);
            return CompletableFuture.completedFuture(updates);
        } catch (Exception e) {
            log.error("checkUpdates via {} failed", mgr, e);
            return CompletableFuture.completedFuture(Map.of());
        }
    }

    private byte[] runAndCapture(SearchService.Cmd cmd) throws IOException, InterruptedException {
        List<String> command = new ArrayList<>(cmd.args().size() + 1);
        command.add(cmd.binary());
        command.addAll(cmd.args());

        ProcessBuilder pb = new ProcessBuilder(command);
        pb.redirectError(ProcessBuilder.Redirect.DISCARD);
        Process p = pb.start();
        byte[] out = p.getInputStream().readAllBytes();
        p.waitFor();
        return out;
    }
}
