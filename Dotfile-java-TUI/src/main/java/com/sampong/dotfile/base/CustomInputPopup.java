package com.sampong.dotfile.base;

import dev.tamboui.toolkit.event.EventResult;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.component.Popups;

import static dev.tamboui.toolkit.Toolkit.text;

/** Custom package-id entry popup. Editing + install wiring lands in Phase 7/9. */
public record CustomInputPopup(StringBuilder value) implements Popup {

    public CustomInputPopup() {
        this(new StringBuilder());
    }

    @Override
    public FeatureView view() {
        return st -> Popups.overlay("Custom package id", text(value + "▌"));
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
