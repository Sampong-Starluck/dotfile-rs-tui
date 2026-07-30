package com.sampong.dotfile.base;

import dev.tamboui.toolkit.event.EventResult;
import com.sampong.dotfile.ui.component.Popups;

import static dev.tamboui.toolkit.Toolkit.text;

/**
 * {@code ?} help overlay. Any key closes it (fleshed out with real bindings in Phase 10).
 * <p>
 * Lives in {@code base} (not its own feature package) because {@code permits} on a sealed
 * type requires same-package subtypes when the project has no {@code module-info.java}
 * (unnamed module) — see FEATURE-PARITY.md deviations.
 */
public record HelpPopup() implements Popup {

    @Override
    public FeatureView view() {
        return st -> Popups.overlay("Help",
                text("1-4 jump - Tab cycle - j/k move - space select").dim(),
                text("d action - / search - ? help - q quit").dim(),
                text("Press any key to close").cyan());
    }

    @Override
    public KeyController controller() {
        return (key, st) -> {
            st.popup = null;
            return EventResult.HANDLED;
        };
    }
}
