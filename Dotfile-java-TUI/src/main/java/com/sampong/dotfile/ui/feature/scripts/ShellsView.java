package com.sampong.dotfile.ui.feature.scripts;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.ui.state.AppState;
import com.sampong.dotfile.ui.component.Panels;

import static dev.tamboui.toolkit.Toolkit.text;

/** {@code [4]-Shells}: shell detect/deploy status list. Lands in Phase 8. */
public class ShellsView implements FeatureView {

    @Override
    public Element render(AppState st) {
        Element content = text("Shells — Phase 8").dim();
        return Panels.framed(4, PanelId.SHELLS.elementId(), "Shells", content);
    }
}
