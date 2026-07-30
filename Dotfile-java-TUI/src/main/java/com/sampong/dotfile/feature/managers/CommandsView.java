package com.sampong.dotfile.feature.managers;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.state.AppState;

import static dev.tamboui.toolkit.Toolkit.text;

/**
 * MAIN/COMMANDS: per-manager command cheat sheet. Table content lands in Phase 6.
 * <p>
 * Returns bare content — {@code ui.TuiApp} wraps every main-view body in the single
 * {@code panel-main} frame (DRY: one place owns the MAIN panel's id/border/focus).
 */
public class CommandsView implements FeatureView {

    @Override
    public Element render(AppState st) {
        return st.platform.selectedManager()
                .<Element>map(pm -> text("Commands for " + pm.label() + " — Phase 6").dim())
                .orElse(text("No active manager selected").dim());
    }
}
