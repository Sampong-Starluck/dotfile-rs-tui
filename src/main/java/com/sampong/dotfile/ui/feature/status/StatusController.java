package com.sampong.dotfile.ui.feature.status;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.ui.state.AppState;
import com.sampong.dotfile.ui.Keys;

/** Display-only, except passing through command-table scroll (Main previews the same table as Managers). */
public class StatusController implements KeyController {

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        if (Keys.isPageUp(key) || key.isChar('K')) {
            st.platform.scrollCommands(-1);
            return EventResult.HANDLED;
        }
        if (Keys.isPageDown(key) || key.isChar('J')) {
            st.platform.scrollCommands(1);
            return EventResult.HANDLED;
        }
        return EventResult.UNHANDLED;
    }
}
