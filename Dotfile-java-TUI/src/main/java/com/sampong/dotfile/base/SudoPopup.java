package com.sampong.dotfile.base;

import dev.tamboui.toolkit.Toolkit;
import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.toolkit.elements.Column;
import dev.tamboui.widgets.input.TextInputState;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.component.Popups;

import java.util.List;
import java.util.function.Consumer;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.text;

/**
 * sudo password entry for non-interactive managers needing elevation (Phase 9, port of
 * {@code app_tab.rs}'s sudo-prompt branch). {@code onSubmit} receives the entered password on
 * Enter; the caller ({@code InstallController}) owns command-prefixing + starting the stream —
 * this popup makes no service calls itself.
 */
public record SudoPopup(String mgr, List<String> displayNames, TextInputState password, Consumer<String> onSubmit)
        implements Popup {

    public SudoPopup(String mgr, List<String> displayNames, Consumer<String> onSubmit) {
        this(mgr, displayNames, new TextInputState(), onSubmit);
    }

    @Override
    public FeatureView view() {
        return st -> {
            Column lines = column();
            lines.add(text("⚠ " + mgr + " requires sudo").yellow());
            for (String name : displayNames) {
                lines.add(text("  " + name));
            }
            lines.add(text(""));
            lines.add(text("•".repeat(password.length()) + "▌"));
            lines.add(text("enter: run · esc: cancel").dim());
            return Popups.overlay("Sudo password", lines);
        };
    }

    @Override
    public KeyController controller() {
        return (key, st) -> {
            if (Keys.isEsc(key)) {
                st.popup = null;
                return EventResult.HANDLED;
            }
            if (Keys.isEnter(key)) {
                String value = password.text();
                if (!value.isEmpty()) {
                    st.popup = null;
                    onSubmit.accept(value);
                }
                return EventResult.HANDLED;
            }
            Toolkit.handleTextInputKey(password, key);
            return EventResult.HANDLED;
        };
    }
}
