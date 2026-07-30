package com.sampong.dotfile.base;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.ui.state.AppState;

/** V: a pure function of state to a fluent Element tree. No side effects. */
public interface FeatureView {
    Element render(AppState st);

    /** Border title for the panel this view renders into; MAIN views override for context. */
    default String title(AppState st) {
        return "Main";
    }
}
