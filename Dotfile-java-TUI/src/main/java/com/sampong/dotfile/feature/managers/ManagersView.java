package com.sampong.dotfile.feature.managers;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.state.AppState;
import com.sampong.dotfile.ui.component.Lists;
import com.sampong.dotfile.ui.component.Panels;

import static dev.tamboui.toolkit.Toolkit.spinner;
import static dev.tamboui.toolkit.Toolkit.text;

/** {@code [2]-Package managers}: spinner while detecting, else a selectable list with an active marker. */
public class ManagersView implements FeatureView {

    @Override
    public Element render(AppState st) {
        Element content;
        if (st.platform.detecting) {
            content = spinner("detecting…");
        } else if (st.platform.packageManagers.isEmpty()) {
            content = text("none found").dim();
        } else {
            boolean focused = st.focused == PanelId.MANAGERS;
            content = Lists.selectable(st.platform.packageManagers, st.platform.managersCursor, focused,
                    pm -> text((pm == st.platform.selectedManager().orElse(null) ? "● " : "  ") + pm.label()));
        }
        return Panels.framed(2, PanelId.MANAGERS.elementId(), "Package managers", content);
    }
}
