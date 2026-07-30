package com.sampong.dotfile.ui.feature.catalog;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.ui.state.AppState;
import com.sampong.dotfile.ui.component.Panels;

import static dev.tamboui.toolkit.Toolkit.text;

/** {@code [3]-Sections}: catalog section list. Loading + navigation land in Phase 7. */
public class SectionsView implements FeatureView {

    @Override
    public Element render(AppState st) {
        Element content = text("Sections — Phase 7").dim();
        return Panels.framed(3, PanelId.SECTIONS.elementId(), "Sections", content);
    }
}
