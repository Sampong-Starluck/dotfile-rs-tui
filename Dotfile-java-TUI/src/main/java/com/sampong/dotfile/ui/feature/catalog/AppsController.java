package com.sampong.dotfile.ui.feature.catalog;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyCode;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.model.AppEntry;
import com.sampong.dotfile.model.AppSection;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.service.CommandPlanner;
import com.sampong.dotfile.service.PackageQueryService;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.state.AppState;
import lombok.RequiredArgsConstructor;

import java.util.List;

/** MAIN/APPS key handling: cursor, Space-select, popups, jump to Installed, Esc back to Sections. */
@RequiredArgsConstructor
public class AppsController implements KeyController {

    private final CommandPlanner commandPlanner;
    private final PackageQueryService packageQueryService;

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        List<AppSection> apps = st.catalog.apps;
        AppSection section = apps != null && !apps.isEmpty() && st.catalog.sectionCursor < apps.size()
                ? apps.get(st.catalog.sectionCursor) : null;
        int appCount = section != null ? section.apps().size() : 0;

        if (Keys.isUp(key)) {
            if (appCount > 0) {
                st.catalog.appCursor = Math.floorMod(st.catalog.appCursor - 1, appCount);
            }
            return EventResult.HANDLED;
        }
        if (Keys.isDown(key)) {
            if (appCount > 0) {
                st.catalog.appCursor = Math.floorMod(st.catalog.appCursor + 1, appCount);
            }
            return EventResult.HANDLED;
        }
        if (key.isChar(' ')) {
            if (section != null && st.catalog.appCursor < appCount) {
                AppEntry entry = section.apps().get(st.catalog.appCursor);
                if (!st.catalog.selectedIds.remove(entry.id())) {
                    st.catalog.selectedIds.add(entry.id());
                }
            }
            return EventResult.HANDLED;
        }
        if (key.isChar('/')) {
            CatalogActions.openSearchPopup(st, packageQueryService);
            return EventResult.HANDLED;
        }
        if (key.isChar('c')) {
            CatalogActions.openCustomPopup(st);
            return EventResult.HANDLED;
        }
        if (key.isChar('l')) {
            CatalogActions.openInstalledView(st, packageQueryService);
            return EventResult.HANDLED;
        }
        if (key.isChar('d')) {
            CatalogActions.openInstallConfirm(st, commandPlanner);
            return EventResult.HANDLED;
        }
        if (Keys.isEsc(key) || key.code() == KeyCode.LEFT) {
            st.requestFocus(PanelId.SECTIONS);
            return EventResult.HANDLED;
        }
        return EventResult.UNHANDLED;
    }
}
