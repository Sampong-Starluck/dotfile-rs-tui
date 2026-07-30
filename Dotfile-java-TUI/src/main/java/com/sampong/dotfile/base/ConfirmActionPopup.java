package com.sampong.dotfile.base;

import dev.tamboui.toolkit.event.EventResult;
import com.sampong.dotfile.ui.component.Popups;
import org.jspecify.annotations.Nullable;

import java.util.List;

/** Lazygit-style confirm dialog: lists the exact commands, y/enter run, n/esc cancel (Phase 7/9). */
public record ConfirmActionPopup(String title, List<String> commands, @Nullable Runnable onConfirm) implements Popup {

    @Override
    public FeatureView view() {
        return st -> Popups.confirm(title, commands, "Run", "Cancel");
    }

    @Override
    public KeyController controller() {
        return (key, st) -> {
            if (key.isChar('y') || key.isConfirm()) {
                st.popup = null;
                if (onConfirm != null) {
                    onConfirm.run();
                }
                return EventResult.HANDLED;
            }
            if (key.isChar('n') || key.isCancel()) {
                st.popup = null;
                return EventResult.HANDLED;
            }
            return EventResult.HANDLED;
        };
    }
}
