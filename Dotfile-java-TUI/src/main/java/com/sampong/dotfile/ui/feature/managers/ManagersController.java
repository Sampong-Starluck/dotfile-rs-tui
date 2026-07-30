package com.sampong.dotfile.ui.feature.managers;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.ui.state.AppState;
import com.sampong.dotfile.ui.Keys;

/** j/k move the cursor; enter/space activates the highlighted manager (cross-feature reset). */
public class ManagersController implements KeyController {

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        if (Keys.isUp(key)) {
            st.platform.moveCursor(-1);
            return EventResult.HANDLED;
        }
        if (Keys.isDown(key)) {
            st.platform.moveCursor(1);
            return EventResult.HANDLED;
        }
        if (Keys.isEnter(key) || key.isChar(' ')) {
            st.activateManager(st.platform.managersCursor);
            return EventResult.HANDLED;
        }
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
