package com.sampong.dotfile.ui.feature.catalog;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.AppSection;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.ui.component.Lists;
import com.sampong.dotfile.ui.component.Panels;
import com.sampong.dotfile.ui.state.AppState;

import java.util.List;

import static dev.tamboui.toolkit.Toolkit.text;

/** {@code [3]-Sections}: catalog section list, filtered for the active manager. */
public class SectionsView implements FeatureView {

    @Override
    public Element render(AppState st) {
        List<AppSection> apps = st.catalog.apps;
        Element content;
        if (apps == null || apps.isEmpty()) {
            content = text("no apps for " + st.platform.activeBinary()).dim();
        } else {
            boolean focused = st.focused == PanelId.SECTIONS;
            content = Lists.selectable(apps, st.catalog.sectionCursor, focused, s -> text(s.section()));
        }
        return Panels.framed(3, PanelId.SECTIONS.elementId(), "Sections", content);
    }
}
