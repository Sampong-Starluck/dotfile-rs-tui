package com.sampong.dotfile.ui;

import dev.tamboui.toolkit.app.ToolkitApp;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Column;
import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;

import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.base.HelpPopup;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.model.AppSection;
import com.sampong.dotfile.model.SearchResult;
import com.sampong.dotfile.ui.feature.catalog.AppsController;
import com.sampong.dotfile.ui.feature.catalog.AppsView;
import com.sampong.dotfile.ui.feature.catalog.CatalogActions;
import com.sampong.dotfile.ui.feature.catalog.SectionsController;
import com.sampong.dotfile.ui.feature.catalog.SectionsView;
import com.sampong.dotfile.ui.feature.install.InstallController;
import com.sampong.dotfile.ui.feature.install.InstallLogBridge;
import com.sampong.dotfile.ui.feature.installed.InstalledController;
import com.sampong.dotfile.ui.feature.installed.InstalledView;
import com.sampong.dotfile.ui.feature.managers.CommandsView;
import com.sampong.dotfile.ui.feature.managers.ManagersController;
import com.sampong.dotfile.ui.feature.managers.ManagersView;
import com.sampong.dotfile.ui.feature.scripts.ShellInfoView;
import com.sampong.dotfile.ui.feature.scripts.ScriptsController;
import com.sampong.dotfile.ui.feature.scripts.ShellsView;
import com.sampong.dotfile.ui.feature.search.SearchController;
import com.sampong.dotfile.ui.feature.search.SearchResultsView;
import com.sampong.dotfile.ui.feature.status.StatusController;
import com.sampong.dotfile.ui.feature.status.StatusView;
import com.sampong.dotfile.model.MainView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.service.AppCatalogService;
import com.sampong.dotfile.service.CommandPlanner;
import com.sampong.dotfile.service.InstallExecutionService;
import com.sampong.dotfile.service.OsService;
import com.sampong.dotfile.service.PackageQueryService;
import com.sampong.dotfile.service.PlatformQueryService;
import com.sampong.dotfile.service.ScriptService;
import com.sampong.dotfile.service.SystemService;
import com.sampong.dotfile.ui.state.AppState;
import com.sampong.dotfile.ui.component.HintBar;
import com.sampong.dotfile.ui.component.Panels;
import com.sampong.dotfile.ui.layout.LazygitLayout;

import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import org.springframework.stereotype.Component;

import java.util.EnumMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

import static dev.tamboui.toolkit.Toolkit.stack;

/**
 * The ONE entry point: extends {@code ToolkitApp}, owns the root {@code render()},
 * global key routing, and panel focus <-> {@link PanelId} sync.
 * <p>
 * Feature views/controllers are plain classes (not Spring beans, per PLAN.md 5.5) registered
 * once here as an {@code EnumMap} — adding a feature never edits routing internals, only these
 * registration maps (OCP).
 */
@Component
@Slf4j
@RequiredArgsConstructor
public class TuiApp extends ToolkitApp {

    private final OsService osService;
    private final PlatformQueryService platformQueryService;
    private final AppCatalogService appCatalogService;
    private final CommandPlanner commandPlanner;
    private final PackageQueryService packageQueryService;
    private final ScriptService scriptService;
    private final SystemService systemService;
    private final InstallExecutionService installExecutionService;
    private final InstallLogBridge installLogBridge;

    private final Map<PanelId, FeatureView> panelViews = buildPanelViews();
    private final Map<MainView, FeatureView> mainViews = buildMainViews();

    /** Populated in {@link #onStart()} — construction needs {@code commandPlanner}/{@code packageQueryService},
     *  which aren't set yet during field-initializer evaluation (they're assigned in the constructor body,
     *  which runs after field initializers per JLS 12.5). */
    private final Map<PanelId, KeyController> panelControllers = new EnumMap<>(PanelId.class);
    private final Map<MainView, KeyController> mainControllers = new EnumMap<>(MainView.class);

    private final AppState st = new AppState();
    private boolean platformDetectStarted;

    /** Set when {@link #frameTick()} sees a queued external command and calls {@link #quit()};
     *  {@link #run()} checks it after the runner returns to decide whether to hand off to
     *  {@link ExternalRunner} and loop, or whether this was a real quit (Phase 9 §9.5). */
    private boolean externalRunPending;

    @Override
    protected void onStart() {
        long start = System.nanoTime();
        st.platform.os = osService.detect();
        log.debug("startup: OS detect took {}ms", (System.nanoTime() - start) / 1_000_000);

        installLogBridge.setRenderWaker(this::requestRender);
        InstallController installController = new InstallController(
                systemService, installExecutionService, installLogBridge, commandPlanner, packageQueryService);

        panelControllers.put(PanelId.STATUS, new StatusController());
        panelControllers.put(PanelId.MANAGERS, new ManagersController());
        panelControllers.put(PanelId.SECTIONS, new SectionsController(commandPlanner, packageQueryService, installController));
        panelControllers.put(PanelId.SHELLS, new ScriptsController(scriptService));

        mainControllers.put(MainView.APPS, new AppsController(commandPlanner, packageQueryService, installController));
        mainControllers.put(MainView.INSTALLED, new InstalledController(commandPlanner, packageQueryService, installController));
        mainControllers.put(MainView.SEARCH_RESULTS, new SearchController(commandPlanner, packageQueryService, installController));
    }

    /**
     * Overridden (not the inherited {@code ToolkitApp.run()}) to implement the suspend-TUI
     * trampoline (PLAN.md phase-09 §9.5): {@code ToolkitRunner} exposes no pause/resume API, so
     * closing and recreating it is the correct implementation (see phase-09 plan note). Each
     * {@code super.run()} call owns one {@code ToolkitRunner} lifecycle end-to-end (including a
     * fresh {@code onStart()}, which is idempotent — {@link #platformDetectStarted} guards the
     * one-shot async platform detect from re-firing); when it returns because {@link
     * #frameTick()} requested an external run, the terminal is already fully restored, so {@link
     * ExternalRunner} can safely use {@code System.out}/{@code System.in} before the loop opens a
     * new runner.
     */
    @Override
    public void run() throws Exception {
        while (true) {
            super.run();
            if (!externalRunPending) {
                return;
            }
            externalRunPending = false;
            ExternalRunner.run(st);
        }
    }

    @Override
    protected Element render() {
        frameTick();

        FeatureView mainView = mainViews.get(st.mainView);
        Column root = LazygitLayout.frame(
                panelViews.get(PanelId.STATUS).render(st),
                panelViews.get(PanelId.MANAGERS).render(st),
                panelViews.get(PanelId.SECTIONS).render(st),
                panelViews.get(PanelId.SHELLS).render(st),
                Panels.framed(PanelId.MAIN.elementId(), mainView.title(st), mainView.render(st)),
                HintBar.of(Bindings.forState(st)),
                st.platform.packageManagers.size(),
                shellCount());

        Element withKeys = root.onKeyEvent(this::handleGlobal);
        return st.popup == null ? withKeys : stack(withKeys, st.popup.view().render(st));
    }

    /** Per-render upkeep: focus requests, lazy platform/catalog loads, future drains (PLAN.md 5.6/7.1). */
    private void frameTick() {
        applyPendingFocus();
        syncFocus();

        if (!platformDetectStarted) {
            platformDetectStarted = true;
            long start = System.nanoTime();
            st.platform.detectFuture = platformQueryService.detectManagers();
            st.platform.detectFuture
                    .thenAccept(managers -> runner().runOnRenderThread(() -> {
                        st.platform.packageManagers = managers;
                        st.platform.detecting = false;
                        log.debug("startup: manager list landed after {}ms",
                                (System.nanoTime() - start) / 1_000_000);
                    }))
                    .thenRun(this::requestRender);
        }

        catalogTick();
        scriptsTick();
        drainInstallLog();

        // External-command trampoline (Phase 9 §9.5): quit() unwinds this ToolkitRunner cleanly;
        // run()'s override hands off to ExternalRunner once the terminal is fully restored.
        if (!st.install.runExternal.isEmpty() && !externalRunPending) {
            externalRunPending = true;
            quit();
        }
    }

    /** Drains streamed install/remove log lines published via {@code InstallLogEvent} ->
     *  {@code InstallLogBridge} into the state the popup renders (PLAN.md §6/§9.2), plus the
     *  latest parsed download/install progress reading (PLAN.md phase-12 §12.1). */
    private void drainInstallLog() {
        var queue = st.install.logQueue;
        if (queue == null) {
            return;
        }
        String line;
        while ((line = queue.poll()) != null) {
            st.install.log.add(line);
        }
        var progress = installLogBridge.currentProgress();
        st.install.progressActive = progress != null;
        st.install.progressLabel = progress == null ? null : progress.label();
        st.install.progressDownloaded = progress == null ? null : progress.downloadedText();
        st.install.progressTotal = progress == null ? null : progress.totalText();
        st.install.progressPercent = progress == null ? 0 : progress.percent();
    }

    /** Lazy catalog load + search/installed future drains + one-time installed auto-load (PLAN.md 7.1).
     *  The load waits out platform detection: filtering earlier would lock {@code st.catalog.apps} in
     *  against {@code activeBinary()}'s "unknown" placeholder, and nothing re-triggers it afterward
     *  (unlike an explicit manager switch, which goes through {@code AppState#activateManager}'s
     *  {@code catalog.reset()}). */
    private void catalogTick() {
        if (st.catalog.apps == null && !st.platform.detecting) {
            long start = System.nanoTime();
            List<AppSection> raw = appCatalogService.readAppsJson();
            st.catalog.apps = appCatalogService.filterByPlatform(raw, osService.osKey(), st.platform.activeBinary());
            log.debug("startup: apps.json load+filter took {}ms", (System.nanoTime() - start) / 1_000_000);
        }

        if (st.search.future != null && st.search.future.isDone()) {
            st.search.results = joinOrEmpty(st.search.future);
            st.search.loading = false;
            st.search.cursor = 0;
            st.search.future = null;
            st.mainView = MainView.SEARCH_RESULTS;
        }

        if (st.installed.future != null && st.installed.future.isDone()) {
            st.installed.packages = joinOrEmpty(st.installed.future);
            st.installed.names.clear();
            st.installed.packages.forEach(pkg -> st.installed.names.add(pkg.name()));
            st.installed.loading = false;
            st.installed.cursor = 0;
            st.installed.future = null;
        }

        if (st.installed.updatesFuture != null && st.installed.updatesFuture.isDone()) {
            st.installed.updates = joinOrEmptyMap(st.installed.updatesFuture);
            st.installed.updatesFuture = null;
        }

        if (!st.installed.autoLoaded && !st.installed.loading && !st.platform.detecting) {
            CatalogActions.refreshInstalled(st, packageQueryService);
            log.debug("triggered background installed-list load for mgr={}", st.platform.activeBinary());
        }
    }

    /** Lazy shells-panel load (PLAN.md phase-08 §8.3) — kept out of {@code ShellsView}/{@code
     *  ShellInfoView} since views are pure/zero-service-call (PLAN.md §4 SRP); fully synchronous
     *  filesystem reads, so unlike {@link #catalogTick()} there's no async future to wait on. */
    private void scriptsTick() {
        if (st.scripts.shells == null) {
            long start = System.nanoTime();
            st.scripts.shells = scriptService.loadShellStatuses();
            st.scripts.explicitPrimaryShell = scriptService.readConfig().primaryShell();
            st.scripts.primaryShell = scriptService.effectivePrimaryShell().orElse(null);
            log.debug("startup: shell statuses load took {}ms", (System.nanoTime() - start) / 1_000_000);
        }
    }

    private static List<SearchResult> joinOrEmpty(CompletableFuture<List<SearchResult>> future) {
        try {
            return future.join();
        } catch (Exception e) {
            log.warn("query future failed", e);
            return List.of();
        }
    }

    private static Map<String, String> joinOrEmptyMap(CompletableFuture<Map<String, String>> future) {
        try {
            return future.join();
        } catch (Exception e) {
            log.warn("updates-check future failed", e);
            return Map.of();
        }
    }

    /** Wakes the runner so background completions get picked up promptly (ticks also cover this). */
    private void requestRender() {
        if (runner() != null) {
            runner().runOnRenderThread(() -> { });
        }
    }

    /** Applies a controller-requested focus jump (e.g. Enter-into-MAIN) through the toolkit's own
     *  focus manager, before {@link #syncFocus()} reads it back — see {@code AppState.pendingFocus}. */
    private void applyPendingFocus() {
        if (st.pendingFocus != null) {
            focusPanel(st.pendingFocus);
            st.pendingFocus = null;
        }
    }

    /** Keeps {@code st.focused}/{@code st.mainView} in sync with the toolkit's own focus state. */
    private void syncFocus() {
        if (runner() == null) {
            return;
        }
        String focusedId = runner().focusManager().focusedId();
        PanelId resolved = null;
        for (PanelId p : PanelId.values()) {
            if (p.elementId().equals(focusedId)) {
                resolved = p;
                break;
            }
        }
        if (resolved == null || resolved == st.focused) {
            return;
        }
        st.focused = resolved;
        switch (resolved) {
            case STATUS, MANAGERS -> st.mainView = MainView.COMMANDS;
            case SECTIONS -> {
                if (st.mainView != MainView.INSTALLED && st.mainView != MainView.SEARCH_RESULTS) {
                    st.mainView = MainView.APPS;
                }
            }
            case SHELLS -> st.mainView = MainView.SHELL_INFO;
            case MAIN -> {
                // no mainView change
            }
        }
        log.debug("focus -> {}", resolved);
    }

    private void focusPanel(PanelId id) {
        if (runner() != null) {
            runner().focusManager().setFocus(id.elementId());
        }
    }

    private EventResult handleGlobal(KeyEvent key) {
        if (st.popup != null) {
            return st.popup.controller().handleKey(key, st);
        }
        if (key.isChar('?')) {
            st.popup = new HelpPopup();
            return EventResult.HANDLED;
        }
        if (key.isChar('q')) {
            quit();
            return EventResult.HANDLED;
        }
        int digit = Keys.digitOf(key);
        if (digit >= 1 && digit <= 4) {
            focusPanel(PanelId.values()[digit - 1]);
            return EventResult.HANDLED;
        }
        KeyController controller = st.focused == PanelId.MAIN
                ? mainControllers.get(st.mainView)
                : panelControllers.get(st.focused);
        return controller != null ? controller.handleKey(key, st) : EventResult.UNHANDLED;
    }

    private int shellCount() {
        return st.scripts.shells != null ? st.scripts.shells.size() : 0;
    }

    private static Map<PanelId, FeatureView> buildPanelViews() {
        Map<PanelId, FeatureView> m = new EnumMap<>(PanelId.class);
        m.put(PanelId.STATUS, new StatusView());
        m.put(PanelId.MANAGERS, new ManagersView());
        m.put(PanelId.SECTIONS, new SectionsView());
        m.put(PanelId.SHELLS, new ShellsView());
        return m;
    }

    private static Map<MainView, FeatureView> buildMainViews() {
        Map<MainView, FeatureView> m = new EnumMap<>(MainView.class);
        m.put(MainView.COMMANDS, new CommandsView());
        m.put(MainView.APPS, new AppsView());
        m.put(MainView.INSTALLED, new InstalledView());
        m.put(MainView.SEARCH_RESULTS, new SearchResultsView());
        m.put(MainView.SHELL_INFO, new ShellInfoView());
        return m;
    }
}
