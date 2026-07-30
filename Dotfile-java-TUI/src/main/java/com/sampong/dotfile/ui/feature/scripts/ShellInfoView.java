package com.sampong.dotfile.ui.feature.scripts;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.ui.state.AppState;

import static dev.tamboui.toolkit.Toolkit.text;

/** MAIN/SHELL_INFO: shell info + action log. Lands in Phase 8. */
public class ShellInfoView implements FeatureView {

    @Override
    public Element render(AppState st) {
        return text("Shell info — Phase 8").dim();
    }
}
