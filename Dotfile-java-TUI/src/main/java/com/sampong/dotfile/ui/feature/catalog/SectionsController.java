package com.sampong.dotfile.ui.feature.catalog;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.ui.state.AppState;

/** Section navigation + Enter-into-MAIN lands in Phase 7. */
public class SectionsController implements KeyController {

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        return EventResult.UNHANDLED;
    }
}
