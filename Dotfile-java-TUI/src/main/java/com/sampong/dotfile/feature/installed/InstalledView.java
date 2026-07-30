package com.sampong.dotfile.feature.installed;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.state.AppState;

import static dev.tamboui.toolkit.Toolkit.text;

/** MAIN/INSTALLED: installed-packages list + remove flow. Lands in Phase 7. */
public class InstalledView implements FeatureView {

    @Override
    public Element render(AppState st) {
        return text("Installed — Phase 7").dim();
    }
}
