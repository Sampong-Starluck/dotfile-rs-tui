package com.sampong.dotfile.ui.feature.search;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.model.MainView;
import com.sampong.dotfile.service.CommandPlanner;
import com.sampong.dotfile.service.PackageQueryService;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.feature.catalog.CatalogActions;
import com.sampong.dotfile.ui.state.AppState;
import lombok.RequiredArgsConstructor;

/** MAIN/SEARCH_RESULTS key handling: cursor, Space-select, reopen search, confirm install, Esc back to Apps. */
@RequiredArgsConstructor
public class SearchController implements KeyController {

    private final CommandPlanner commandPlanner;
    private final PackageQueryService packageQueryService;

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        int count = st.search.results.size();

        if (Keys.isUp(key)) {
            if (count > 0) {
                st.search.cursor = Math.floorMod(st.search.cursor - 1, count);
            }
            return EventResult.HANDLED;
        }
        if (Keys.isDown(key)) {
            if (count > 0) {
                st.search.cursor = Math.floorMod(st.search.cursor + 1, count);
            }
            return EventResult.HANDLED;
        }
        if (key.isChar(' ')) {
            if (st.search.cursor < count) {
                String id = st.search.results.get(st.search.cursor).id();
                if (!st.catalog.selectedIds.remove(id)) {
                    st.catalog.selectedIds.add(id);
                }
            }
            return EventResult.HANDLED;
        }
        if (key.isChar('/')) {
            CatalogActions.openSearchPopup(st, packageQueryService);
            return EventResult.HANDLED;
        }
        if (key.isChar('d')) {
            CatalogActions.openInstallConfirm(st, commandPlanner);
            return EventResult.HANDLED;
        }
        if (Keys.isEsc(key)) {
            st.mainView = MainView.APPS;
            return EventResult.HANDLED;
        }
        return EventResult.UNHANDLED;
    }
}
