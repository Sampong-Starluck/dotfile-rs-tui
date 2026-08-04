package com.sampong.dotfile.ui.feature.managers;

import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Column;
import dev.tamboui.toolkit.elements.Row;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PackageManager;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.ui.state.AppState;
import com.sampong.dotfile.ui.component.Lists;
import com.sampong.dotfile.ui.component.Panels;
import com.sampong.dotfile.ui.component.Responsive;
import com.sampong.dotfile.ui.component.UiText;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.row;
import static dev.tamboui.toolkit.Toolkit.spinner;
import static dev.tamboui.toolkit.Toolkit.text;

/** {@code [2]-Package managers}: spinner while detecting, else a selectable list with an active marker. */
public class ManagersView implements FeatureView {

    private static final int DESCRIPTION_INDENT = 4;

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

    /** {@code clamp(pmCount, 1, 6) * 2 + 2} — every visible row reserves 2 lines (label + description). */
    public static int panelHeight(int pmCount) {
        return Math.min(Math.max(pmCount, 1), 6) * 2 + 2;
    }

    /**
     * The description gets its own indented line below the label instead of competing with it for
     * horizontal space — a human review of {@code mise run dev} found descriptions routinely
     * clipped when squeezed onto the same line as the marker + label, even after widening the
     * side column (PLAN.md 5). {@link UiText#truncate} stays as a safety net for terminals too
     * narrow to fit the description even on its own line.
     */
    private static Column managerRow(PackageManager pm, AppState st, boolean useBinary, int innerWidth) {
        boolean active = st.platform.selectedManager().map(sel -> sel == pm).orElse(false);
        String label = useBinary ? pm.binary() : pm.label();
        Row labelLine = row(active ? text("● ").green() : text("  "), text(label));

        String desc = pm.description();
        if (desc.isEmpty()) {
            return column(labelLine);
        }
        int available = Math.max(innerWidth - DESCRIPTION_INDENT, 0);
        Row descLine = row(text(" ".repeat(DESCRIPTION_INDENT) + UiText.truncate(desc, available)).dim());
        return column(labelLine, descLine);
    }
}
