package com.sampong.dotfile.feature.managers;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.state.AppState;
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
        return EventResult.UNHANDLED;
    }
}
