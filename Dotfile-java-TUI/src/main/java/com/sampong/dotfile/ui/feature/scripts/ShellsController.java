package com.sampong.dotfile.ui.feature.scripts;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.ui.state.AppState;

/** Shell navigation + deploy/undeploy/primary-shell actions land in Phase 8. */
public class ShellsController implements KeyController {

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        return EventResult.UNHANDLED;
    }
}
