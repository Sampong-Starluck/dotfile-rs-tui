package com.sampong.dotfile.base;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.state.AppState;

/** V: a pure function of state to a fluent Element tree. No side effects. */
public interface FeatureView {
    Element render(AppState st);
}
