package com.sampong.dotfile.ui.feature.catalog;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.ui.state.AppState;

import static dev.tamboui.toolkit.Toolkit.text;

/** MAIN/APPS: apps of the highlighted section. Lands in Phase 7. */
public class AppsView implements FeatureView {

    @Override
    public Element render(AppState st) {
        return text("Apps — Phase 7").dim();
    }
}
