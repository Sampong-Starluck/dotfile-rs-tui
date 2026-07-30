package com.sampong.dotfile.feature.status;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.state.AppState;

/** Status panel is display-only. */
public class StatusController implements KeyController {

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        return EventResult.UNHANDLED;
    }
}
