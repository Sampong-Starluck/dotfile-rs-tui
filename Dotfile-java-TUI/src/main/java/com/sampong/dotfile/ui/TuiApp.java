package com.sampong.dotfile.ui;

import dev.tamboui.toolkit.app.ToolkitApp;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Column;
import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.base.HelpPopup;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.ui.feature.catalog.AppsView;
import com.sampong.dotfile.ui.feature.catalog.SectionsController;
import com.sampong.dotfile.ui.feature.catalog.SectionsView;
import com.sampong.dotfile.ui.feature.installed.InstalledView;
import com.sampong.dotfile.ui.feature.managers.CommandsView;
import com.sampong.dotfile.ui.feature.managers.ManagersController;
import com.sampong.dotfile.ui.feature.managers.ManagersView;
import com.sampong.dotfile.ui.feature.scripts.ShellInfoView;
import com.sampong.dotfile.ui.feature.scripts.ShellsController;
import com.sampong.dotfile.ui.feature.scripts.ShellsView;
import com.sampong.dotfile.ui.feature.search.SearchResultsView;
import com.sampong.dotfile.ui.feature.status.StatusController;
import com.sampong.dotfile.ui.feature.status.StatusView;
import com.sampong.dotfile.model.MainView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.service.OsService;
import com.sampong.dotfile.service.PlatformQueryService;
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

import static dev.tamboui.toolkit.Toolkit.stack;

/**
 * The ONE entry point: extends {@code ToolkitApp}, owns the root {@code render()},
 * global key routing, and panel focus <-> {@link PanelId} sync.
 * <p>
 * Feature views/controllers are plain classes (not Spring beans, per PLAN.md 5.5) registered
 * once here as an {@code EnumMap} — adding a feature never edits routing internals, only these
 * three registration maps (OCP).
 */
@Component
@Slf4j
@RequiredArgsConstructor
public class TuiApp extends ToolkitApp {

    private final OsService osService;
    private final PlatformQueryService platformQueryService;

    private final Map<PanelId, FeatureView> panelViews = buildPanelViews();
    private final Map<PanelId, KeyController> panelControllers = buildPanelControllers();
    private final Map<MainView, FeatureView> mainViews = buildMainViews();

    private final AppState st = new AppState();
    private boolean platformDetectStarted;

    @Override
    protected void onStart() {
        long start = System.nanoTime();
        st.platform.os = osService.detect();
        log.debug("startup: OS detect took {}ms", (System.nanoTime() - start) / 1_000_000);
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
                HintBar.of(keyHints()),
                st.platform.packageManagers.size(),
                shellCount());

        Element withKeys = root.onKeyEvent(this::handleGlobal);
        return st.popup == null ? withKeys : stack(withKeys, st.popup.view().render(st));
    }

    /** Per-render upkeep: lazy platform detection kickoff/drain, focus sync (PLAN.md 5.6). */
    private void frameTick() {
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

        // External-command trampoline (Phase 9); until then, log + clear.
        if (!st.install.runExternal.isEmpty()) {
            log.debug("external run requested (Phase 9 not yet wired): {}", st.install.runExternal);
            st.install.runExternal.clear();
        }
    }

    /** Wakes the runner so background completions get picked up promptly (ticks also cover this). */
    private void requestRender() {
        if (runner() != null) {
            runner().runOnRenderThread(() -> { });
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
        KeyController controller = panelControllers.get(st.focused);
        return controller != null ? controller.handleKey(key, st) : EventResult.UNHANDLED;
    }

    private int shellCount() {
        return st.scripts.shells != null ? st.scripts.shells.size() : 0;
    }

    private List<HintBar.Binding> keyHints() {
        return switch (st.focused) {
            case STATUS, MANAGERS -> List.of(
                    new HintBar.Binding("enter", "use manager"),
                    new HintBar.Binding("j/k", "move"),
                    new HintBar.Binding("pgup/pgdn", "scroll commands"),
                    new HintBar.Binding("1-4", "jump"),
                    new HintBar.Binding("?", "help"),
                    new HintBar.Binding("q", "quit"));
            default -> List.of(
                    new HintBar.Binding("1-4", "jump"),
                    new HintBar.Binding("tab", "cycle"),
                    new HintBar.Binding("j/k", "move"),
                    new HintBar.Binding("space", "select"),
                    new HintBar.Binding("?", "help"),
                    new HintBar.Binding("q", "quit"));
        };
    }

    private static Map<PanelId, FeatureView> buildPanelViews() {
        Map<PanelId, FeatureView> m = new EnumMap<>(PanelId.class);
        m.put(PanelId.STATUS, new StatusView());
        m.put(PanelId.MANAGERS, new ManagersView());
        m.put(PanelId.SECTIONS, new SectionsView());
        m.put(PanelId.SHELLS, new ShellsView());
        return m;
    }

    private static Map<PanelId, KeyController> buildPanelControllers() {
        Map<PanelId, KeyController> m = new EnumMap<>(PanelId.class);
        m.put(PanelId.STATUS, new StatusController());
        m.put(PanelId.MANAGERS, new ManagersController());
        m.put(PanelId.SECTIONS, new SectionsController());
        m.put(PanelId.SHELLS, new ShellsController());
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
