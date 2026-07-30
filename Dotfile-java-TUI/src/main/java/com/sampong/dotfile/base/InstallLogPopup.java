package com.sampong.dotfile.base;

import dev.tamboui.toolkit.event.EventResult;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.component.Logs;
import com.sampong.dotfile.ui.component.Popups;

/** Streaming install log + interactive stdin input line (Phase 9). State lives in {@code AppState.install}. */
public record InstallLogPopup() implements Popup {

    @Override
    public FeatureView view() {
        return st -> Popups.overlay("Install log", Logs.colored(st.install.log, 20));
    }

    @Override
    public KeyController controller() {
        return (key, st) -> {
            if (Keys.isEsc(key)) {
                st.popup = null;
                return EventResult.HANDLED;
            }
            return EventResult.HANDLED;
        };
    }
}
