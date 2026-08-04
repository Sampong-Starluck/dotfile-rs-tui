package com.sampong.dotfile.base;

import dev.tamboui.toolkit.Toolkit;
import dev.tamboui.toolkit.elements.Column;
import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.widgets.input.TextInputState;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.component.Inputs;
import com.sampong.dotfile.ui.component.Logs;
import com.sampong.dotfile.ui.component.Popups;
import com.sampong.dotfile.ui.component.ProgressBar;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.text;

/**
 * Streaming install/remove log + interactive stdin input line (Phase 9). Log/queues live in
 * {@code AppState.install}; {@code onClose} lets the caller ({@code InstallController}) decide
 * what focus/refresh happens after Esc, so this popup makes no service calls itself.
 */
public record InstallLogPopup(TextInputState input, Runnable onClose) implements Popup {

    public InstallLogPopup(Runnable onClose) {
        this(new TextInputState(), onClose);
    }

    @Override
    public FeatureView view() {
        return st -> {
            Column body = column();
            if (st.install.targetLabel != null) {
                body.add(text(st.install.targetLabel).bold());
            }
            if (st.install.progressActive) {
                body.add(ProgressBar.render(st.install.progressLabel, st.install.progressDownloaded,
                        st.install.progressTotal, st.install.progressPercent));
            }
            body.add(Logs.colored(st.install.log, 20));
            String title = switch (st.install.kind) {
                case REMOVE -> "⬇ Removing";
                case UPDATE -> "⬆ Updating";
                case INSTALL -> "⬇ Installing";
                case null -> "⬇ Installing";
            };
            return Popups.overlay(title, body,
                    Inputs.line(input, "").focusable(false).cursorRequiresFocus(false));
        };
    }

    @Override
    public KeyController controller() {
        return (key, st) -> {
            if (Keys.isEsc(key)) {
                st.popup = null;
                st.install.logQueue = null;
                st.install.stdinQueue = null;
                st.install.log.clear();
                st.install.targetLabel = null;
                st.install.progressActive = false;
                st.catalog.selectedIds.clear();
                onClose.run();
                return EventResult.HANDLED;
            }
            if (Keys.isEnter(key)) {
                String text = input.text();
                if (!text.isEmpty()) {
                    st.install.log.add("> " + text);
                    var stdinQueue = st.install.stdinQueue;
                    if (stdinQueue != null) {
                        stdinQueue.offer(text);
                    }
                    input.clear();
                }
                return EventResult.HANDLED;
            }
            Toolkit.handleTextInputKey(input, key);
            return EventResult.HANDLED;
        };
    }
}
