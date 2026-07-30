package com.sampong.dotfile.base;

import dev.tamboui.toolkit.Toolkit;
import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.widgets.input.TextInputState;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.component.Inputs;
import com.sampong.dotfile.ui.component.Popups;

import static dev.tamboui.toolkit.Toolkit.text;

/** Custom package-id entry popup: Enter/Space commits the trimmed buffer into {@code st.catalog.selectedIds}. */
public record CustomInputPopup(TextInputState value) implements Popup {

    @Override
    public FeatureView view() {
        return st -> Popups.overlay("Custom package id",
                Inputs.line(value, "").focusable(false).cursorRequiresFocus(false),
                text("enter/space: add · esc: cancel").dim());
    }

    @Override
    public KeyController controller() {
        return (key, st) -> {
            if (Keys.isEsc(key)) {
                st.popup = null;
                return EventResult.HANDLED;
            }
            if (Keys.isEnter(key) || key.isChar(' ')) {
                String trimmed = value.text().trim();
                if (!trimmed.isEmpty()) {
                    st.catalog.selectedIds.add(trimmed);
                }
                st.popup = null;
                return EventResult.HANDLED;
            }
            Toolkit.handleTextInputKey(value, key);
            return EventResult.HANDLED;
        };
    }
}
