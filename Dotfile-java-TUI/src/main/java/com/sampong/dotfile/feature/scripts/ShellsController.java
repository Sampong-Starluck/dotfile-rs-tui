package com.sampong.dotfile.feature.scripts;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.state.AppState;

/** Shell navigation + deploy/undeploy/primary-shell actions land in Phase 8. */
public class ShellsController implements KeyController {

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        return EventResult.UNHANDLED;
    }
}
