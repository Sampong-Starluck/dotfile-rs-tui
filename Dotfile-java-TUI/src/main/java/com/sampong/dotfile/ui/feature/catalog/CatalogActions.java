package com.sampong.dotfile.ui.feature.catalog;

import dev.tamboui.widgets.input.TextInputState;
import com.sampong.dotfile.base.ConfirmActionPopup;
import com.sampong.dotfile.base.CustomInputPopup;
import com.sampong.dotfile.base.SearchInputPopup;
import com.sampong.dotfile.model.AppSection;
import com.sampong.dotfile.model.InstallKind;
import com.sampong.dotfile.model.MainView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.service.CommandPlanner;
import com.sampong.dotfile.service.PackageQueryService;
import com.sampong.dotfile.ui.feature.install.InstallController;
import com.sampong.dotfile.ui.state.AppState;

import java.util.ArrayList;
import java.util.List;

/**
 * Shared actions used by every feature that touches {@code st.catalog.selectedIds} (Sections,
 * Apps, Search results, Installed) — one place owns confirm-popup construction, custom/search
 * popup construction, and the installed-list refresh trigger (SRP/DRY, PLAN.md §7).
 */
public final class CatalogActions {
    private CatalogActions() {
    }

    /** {@code d} on Sections/Apps/Search: build the install confirm popup; no-op if nothing selected. */
    public static void openInstallConfirm(AppState st, CommandPlanner commandPlanner, InstallController installController) {
        String mgr = st.platform.activeBinary();
        List<AppSection> apps = st.catalog.apps != null ? st.catalog.apps : List.of();
        List<String> commands = commandPlanner.buildInstallCommands(apps, st.catalog.selectedIds, mgr);
        if (commands.isEmpty()) {
            return;
        }
        st.popup = new ConfirmActionPopup(
                "Install " + commands.size() + " package(s)?",
                "runs with " + mgr,
                commands,
                () -> installController.start(st, InstallKind.INSTALL, commands));
    }

    /** {@code d} on Installed: build the remove confirm popup; no-op if nothing selected. */
    public static void openRemoveConfirm(AppState st, CommandPlanner commandPlanner, InstallController installController) {
        String mgr = st.platform.activeBinary();
        List<String> commands = commandPlanner.buildRemoveCommands(st.catalog.selectedIds, mgr);
        if (commands.isEmpty()) {
            return;
        }
        st.popup = new ConfirmActionPopup(
                "Remove " + commands.size() + " package(s)?",
                "runs with " + mgr,
                commands,
                () -> installController.start(st, InstallKind.REMOVE, commands));
    }

    /** {@code /}: open the search popup pre-filled with the last query. */
    public static void openSearchPopup(AppState st, PackageQueryService packageQueryService) {
        st.popup = new SearchInputPopup(new TextInputState(st.search.lastQuery),
                query -> startSearch(st, query, packageQueryService));
    }

    /** {@code c}: open the custom package-id popup. */
    public static void openCustomPopup(AppState st) {
        st.popup = new CustomInputPopup(new TextInputState());
    }

    private static void startSearch(AppState st, String query, PackageQueryService packageQueryService) {
        String mgr = st.platform.activeBinary();
        st.search.lastQuery = query;
        st.search.loading = true;
        st.mainView = MainView.SEARCH_RESULTS;
        st.requestFocus(PanelId.MAIN);
        st.search.future = packageQueryService.search(mgr, query);
    }

    /** {@code l} on Sections/Apps: switch to the Installed view and force a fresh query. */
    public static void openInstalledView(AppState st, PackageQueryService packageQueryService) {
        st.catalog.selectedIds.clear();
        st.installed.removeMode = true;
        st.mainView = MainView.INSTALLED;
        st.requestFocus(PanelId.MAIN);
        refreshInstalled(st, packageQueryService);
    }

    /** {@code r} on Installed, the initial {@code l} jump, and the one-time startup auto-load all
     *  funnel through here, so {@code autoLoaded} is set unconditionally: whichever path loads
     *  first, the startup auto-load (PLAN.md §7.1.3) must never fire a redundant second query. */
    public static void refreshInstalled(AppState st, PackageQueryService packageQueryService) {
        st.installed.packages = new ArrayList<>();
        st.installed.names.clear();
        st.installed.loading = true;
        st.installed.autoLoaded = true;
        st.installed.future = packageQueryService.listInstalled(st.platform.activeBinary());
    }
}
