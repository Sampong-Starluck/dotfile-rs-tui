package com.sampong.dotfile.ui.feature.managers;

import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Row;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PackageManager;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.ui.state.AppState;
import com.sampong.dotfile.ui.component.Lists;
import com.sampong.dotfile.ui.component.Panels;
import com.sampong.dotfile.ui.component.Responsive;
import com.sampong.dotfile.ui.component.UiText;

import static dev.tamboui.toolkit.Toolkit.row;
import static dev.tamboui.toolkit.Toolkit.spinner;
import static dev.tamboui.toolkit.Toolkit.text;

/** {@code [2]-Package managers}: spinner while detecting, else a selectable list with an active marker. */
public class ManagersView implements FeatureView {

    @Override
    public Element render(AppState st) {
        Element content;
        if (st.platform.detecting) {
            content = row(spinner(), text(" detecting…").dim());
        } else if (st.platform.packageManagers.isEmpty()) {
            content = text("no package managers detected").dim();
        } else {
            boolean focused = st.focused == PanelId.MANAGERS;
            content = Responsive.of(area -> {
                boolean useBinary = area.width() < 18;
                return Lists.selectable(st.platform.packageManagers, st.platform.managersCursor, focused,
                        pm -> managerRow(pm, st, useBinary, area.width()));
            });
        }
        return Panels.framed(2, PanelId.MANAGERS.elementId(), "Package managers", content);
    }

    /** Truncates the description to whatever room is left after the marker + label, rather than
     *  letting the terminal hard-clip it mid-word — matches the ellipsis convention {@link UiText}
     *  already uses for the catalog/search/installed lists. */
    private static Row managerRow(PackageManager pm, AppState st, boolean useBinary, int innerWidth) {
        boolean active = st.platform.selectedManager().map(sel -> sel == pm).orElse(false);
        String label = useBinary ? pm.binary() : pm.label();
        Row line = row(active ? text("● ").green() : text("  "), text(label));

        int available = innerWidth - (2 + label.length() + 3);
        if (available > 0) {
            String desc = UiText.truncate(pm.description(), available);
            if (!desc.isEmpty()) {
                line.add(text(" — " + desc).dim());
            }
        }
        return line;
    }
}
