package com.sampong.dotfile.base;

import dev.tamboui.toolkit.Toolkit;
import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.widgets.input.TextInputState;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.component.Inputs;
import com.sampong.dotfile.ui.component.Popups;

import java.util.function.Consumer;

import static dev.tamboui.toolkit.Toolkit.text;

/** {@code /} search query popup; {@code onSearch} is invoked with the trimmed query on Enter. */
public record SearchInputPopup(TextInputState query, Consumer<String> onSearch) implements Popup {

    @Override
    public FeatureView view() {
        return st -> Popups.overlay("Search " + st.platform.activeBinary(),
                Inputs.line(query, "").focusable(false).cursorRequiresFocus(false),
                text("enter: search · esc: cancel").dim());
    }

    @Override
    public KeyController controller() {
        return (key, st) -> {
            if (Keys.isEsc(key)) {
                st.popup = null;
                return EventResult.HANDLED;
            }
            if (Keys.isEnter(key)) {
                String trimmed = query.text().trim();
                if (!trimmed.isEmpty()) {
                    st.popup = null;
                    onSearch.accept(trimmed);
                }
                return EventResult.HANDLED;
            }
            Toolkit.handleTextInputKey(query, key);
            return EventResult.HANDLED;
        };
    }
}
