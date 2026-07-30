package com.sampong.dotfile.ui.feature.installed;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.model.MainView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.service.CommandPlanner;
import com.sampong.dotfile.service.PackageQueryService;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.feature.catalog.CatalogActions;
import com.sampong.dotfile.ui.state.AppState;
import lombok.RequiredArgsConstructor;

/** MAIN/INSTALLED key handling: cursor, Space-select, refresh, confirm remove, Esc back to Sections. */
@RequiredArgsConstructor
public class InstalledController implements KeyController {

    private final CommandPlanner commandPlanner;
    private final PackageQueryService packageQueryService;

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        int count = st.installed.packages.size();

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
                String id = st.installed.packages.get(st.installed.cursor).id();
                if (!st.catalog.selectedIds.remove(id)) {
                    st.catalog.selectedIds.add(id);
                }
            }
            return EventResult.HANDLED;
        }
        if (key.isChar('r')) {
            CatalogActions.refreshInstalled(st, packageQueryService);
            return EventResult.HANDLED;
        }
        if (key.isChar('d')) {
            CatalogActions.openRemoveConfirm(st, commandPlanner);
            return EventResult.HANDLED;
        }
        if (Keys.isEsc(key)) {
            st.mainView = MainView.APPS;
            st.installed.removeMode = false;
            st.catalog.selectedIds.clear();
            st.requestFocus(PanelId.SECTIONS);
            return EventResult.HANDLED;
        }
        return EventResult.UNHANDLED;
    }
}
