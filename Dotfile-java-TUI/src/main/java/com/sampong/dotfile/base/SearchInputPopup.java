package com.sampong.dotfile.base;

import dev.tamboui.toolkit.event.EventResult;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.component.Popups;

import static dev.tamboui.toolkit.Toolkit.text;

/** {@code /} search query popup. Query editing wired in Phase 7. */
public record SearchInputPopup(StringBuilder query) implements Popup {

    public SearchInputPopup() {
        this(new StringBuilder());
    }

    @Override
    public FeatureView view() {
        return st -> Popups.overlay("Search", text(query + "▌"));
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
