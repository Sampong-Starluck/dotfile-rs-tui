package com.sampong.dotfile.base;

import dev.tamboui.toolkit.event.EventResult;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.component.Popups;

import static dev.tamboui.toolkit.Toolkit.text;

/** sudo password entry on Linux for interactive-manager installs (Phase 9). */
public record SudoPopup(StringBuilder password) implements Popup {

    public SudoPopup() {
        this(new StringBuilder());
    }

    @Override
    public FeatureView view() {
        return st -> Popups.overlay("Sudo password", text("*".repeat(password.length()) + "▌"));
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
