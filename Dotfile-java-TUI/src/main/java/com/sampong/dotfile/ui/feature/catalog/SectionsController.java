package com.sampong.dotfile.ui.feature.catalog;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyCode;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.model.MainView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.service.CommandPlanner;
import com.sampong.dotfile.service.PackageQueryService;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.feature.install.InstallController;
import com.sampong.dotfile.ui.state.AppState;
import lombok.RequiredArgsConstructor;

/** Section navigation; Enter/→ into MAIN/APPS; {@code / c l d} open the shared catalog actions. */
@RequiredArgsConstructor
public class SectionsController implements KeyController {

    private final CommandPlanner commandPlanner;
    private final PackageQueryService packageQueryService;
    private final InstallController installController;

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        int sectionCount = st.catalog.apps != null ? st.catalog.apps.size() : 0;

        if (Keys.isUp(key)) {
            moveCursor(st, sectionCount, -1);
            return EventResult.HANDLED;
        }
        if (Keys.isDown(key)) {
            moveCursor(st, sectionCount, 1);
            return EventResult.HANDLED;
        }
        if (Keys.isEnter(key) || key.code() == KeyCode.RIGHT || key.isChar(' ')) {
            st.mainView = MainView.APPS;
            st.requestFocus(PanelId.MAIN);
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
            CatalogActions.openInstallConfirm(st, commandPlanner, installController);
            return EventResult.HANDLED;
        }
        return EventResult.UNHANDLED;
    }

    private static void moveCursor(AppState st, int sectionCount, int delta) {
        st.catalog.sectionCursor = sectionCount == 0 ? 0 : Math.floorMod(st.catalog.sectionCursor + delta, sectionCount);
        st.catalog.appCursor = 0;
    }
}
