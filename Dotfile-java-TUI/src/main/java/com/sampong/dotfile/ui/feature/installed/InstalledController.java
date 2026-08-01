package com.sampong.dotfile.ui.feature.installed;

import dev.tamboui.toolkit.Toolkit;
import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.model.MainView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.service.CommandPlanner;
import com.sampong.dotfile.service.PackageQueryService;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.feature.catalog.CatalogActions;
import com.sampong.dotfile.ui.feature.install.InstallController;
import com.sampong.dotfile.ui.state.AppState;
import lombok.RequiredArgsConstructor;

/** MAIN/INSTALLED key handling: cursor, Space-select, refresh, confirm remove, Esc back to
 *  Sections — all operating on {@code st.installed.visiblePackages()} (the fuzzy-filtered view,
 *  Phase 13) rather than the raw list, so actions always match what's on screen. {@code /}
 *  enters filter-editing mode ({@link #handleFilterKey}), where every key except Up/Down/
 *  Enter/Esc becomes filter-query text instead of a list action — matching how real
 *  fuzzy-finders capture the keyboard while their search box is focused. */
@RequiredArgsConstructor
public class InstalledController implements KeyController {

    private final CommandPlanner commandPlanner;
    private final PackageQueryService packageQueryService;
    private final InstallController installController;

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        if (st.installed.filtering) {
            return handleFilterKey(key, st);
        }

        int count = st.installed.visiblePackages().size();

        if (Keys.isUp(key)) {
            if (count > 0) {
                st.installed.cursor = Math.floorMod(st.installed.cursor - 1, count);
            }
            return EventResult.HANDLED;
        }
        if (Keys.isDown(key)) {
            if (count > 0) {
                st.installed.cursor = Math.floorMod(st.installed.cursor + 1, count);
            }
            return EventResult.HANDLED;
        }
        if (key.isChar(' ')) {
            if (st.installed.cursor < count) {
                String id = st.installed.visiblePackages().get(st.installed.cursor).id();
                if (!st.catalog.selectedIds.remove(id)) {
                    st.catalog.selectedIds.add(id);
                }
            }
            return EventResult.HANDLED;
        }
        if (key.isChar('/')) {
            st.installed.filtering = true;
            return EventResult.HANDLED;
        }
        if (key.isChar('r')) {
            CatalogActions.refreshInstalled(st, packageQueryService);
            return EventResult.HANDLED;
        }
        if (key.isChar('d')) {
            CatalogActions.openRemoveConfirm(st, commandPlanner, installController);
            return EventResult.HANDLED;
        }
        if (key.isChar('u')) {
            CatalogActions.openUpdateConfirm(st, commandPlanner, installController);
            return EventResult.HANDLED;
        }
        if (Keys.isEsc(key)) {
            st.mainView = MainView.APPS;
            st.installed.removeMode = false;
            st.installed.filterQuery.clear();
            st.catalog.selectedIds.clear();
            st.requestFocus(PanelId.SECTIONS);
            return EventResult.HANDLED;
        }
        return EventResult.UNHANDLED;
    }

    private EventResult handleFilterKey(KeyEvent key, AppState st) {
        if (Keys.isEsc(key)) {
            st.installed.filtering = false;
            st.installed.filterQuery.clear();
            st.installed.cursor = 0;
            return EventResult.HANDLED;
        }
        if (Keys.isEnter(key)) {
            st.installed.filtering = false;
            return EventResult.HANDLED;
        }
        if (Keys.isUp(key) || Keys.isDown(key)) {
            int count = st.installed.visiblePackages().size();
            if (count > 0) {
                int delta = Keys.isUp(key) ? -1 : 1;
                st.installed.cursor = Math.floorMod(st.installed.cursor + delta, count);
            }
            return EventResult.HANDLED;
        }
        Toolkit.handleTextInputKey(st.installed.filterQuery, key);
        st.installed.cursor = 0;
        return EventResult.HANDLED;
    }
}
