package com.sampong.dotfile.base;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.state.AppState;

/** C: handles a key for its feature. HANDLED stops propagation. */
public interface KeyController {
    EventResult handleKey(KeyEvent key, AppState st);
}
