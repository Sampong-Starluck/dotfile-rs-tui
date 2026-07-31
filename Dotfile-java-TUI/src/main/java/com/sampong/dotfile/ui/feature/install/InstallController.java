package com.sampong.dotfile.ui.feature.install;

import com.sampong.dotfile.base.InstallLogPopup;
import com.sampong.dotfile.base.SudoPopup;
import com.sampong.dotfile.model.InstallKind;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.service.CommandPlanner;
import com.sampong.dotfile.service.InstallExecutionService;
import com.sampong.dotfile.service.PackageQueryService;
import com.sampong.dotfile.service.SystemService;
import com.sampong.dotfile.ui.feature.catalog.CatalogActions;
import com.sampong.dotfile.ui.state.AppState;
import lombok.RequiredArgsConstructor;
import org.jspecify.annotations.Nullable;

import java.util.List;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.LinkedBlockingQueue;

/**
 * Install/remove decision tree — sudo prompt, suspend-TUI trampoline, or straight streaming
 * (port of {@code app_tab.rs}'s install/remove dispatch, PLAN.md phase-09 §9.4). Not a Spring
 * bean: instantiated once in {@code TuiApp.onStart()}, like the other feature controllers.
 */
@RequiredArgsConstructor
public class InstallController {

    private final SystemService systemService;
    private final InstallExecutionService executionService;
    private final InstallLogBridge logBridge;
    private final CommandPlanner commandPlanner;
    private final PackageQueryService packageQueryService;

    public void start(AppState st, InstallKind kind, List<String> commands) {
        if (commands.isEmpty()) {
            return;
        }
        st.install.kind = kind;
        String mgr = st.platform.activeBinary();

        if (systemService.requiresInteractive(mgr)) {
            List<String> cmds = (systemService.requiresSudo(mgr) && !systemService.isRoot())
                    ? commands.stream().map(c -> "sudo " + c).toList()
                    : commands;
            st.install.runExternal.addAll(cmds);
            st.install.runExternalRemoving = (kind == InstallKind.REMOVE);
            return;
        }
        if (systemService.requiresSudo(mgr) && !systemService.isRoot()) {
            List<String> names = commandPlanner.displayNames(
                    st.catalog.apps != null ? st.catalog.apps : List.of(), st.catalog.selectedIds);
            st.popup = new SudoPopup(mgr, names, password -> {
                List<String> prefixed = commands.stream().map(c -> "sudo -S " + c).toList();
                begin(st, prefixed, password);
            });
            return;
        }
        begin(st, commands, null);
    }

    private void begin(AppState st, List<String> commands, @Nullable String sudoPassword) {
        st.install.log.clear();
        st.install.log.add(st.install.kind == InstallKind.REMOVE ? "Starting removal…" : "Starting installation…");
        st.install.logQueue = new ConcurrentLinkedQueue<>();
        st.install.stdinQueue = new LinkedBlockingQueue<>();
        logBridge.attach(st.install.logQueue);
        st.popup = new InstallLogPopup(() -> onClose(st));
        executionService.runStreaming(commands, sudoPassword, st.install.stdinQueue);
    }

    private void onClose(AppState st) {
        logBridge.detach();
        if (st.installed.removeMode) {
            CatalogActions.refreshInstalled(st, packageQueryService);
        } else {
            st.requestFocus(PanelId.SECTIONS);
        }
    }
}
