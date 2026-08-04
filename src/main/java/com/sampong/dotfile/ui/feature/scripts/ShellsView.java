package com.sampong.dotfile.ui.feature.scripts;

import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Row;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.model.ShellStatus;
import com.sampong.dotfile.ui.component.Lists;
import com.sampong.dotfile.ui.component.Panels;
import com.sampong.dotfile.ui.component.Responsive;
import com.sampong.dotfile.ui.state.AppState;

import java.util.List;

import static dev.tamboui.toolkit.Toolkit.row;
import static dev.tamboui.toolkit.Toolkit.text;

/** {@code [4]-Shells}: shell detect/deploy status list. */
public class ShellsView implements FeatureView {

    private static final int NARROW_WIDTH = 18;

    @Override
    public Element render(AppState st) {
        List<ShellStatus> shells = st.scripts.shells;
        Element content;
        if (shells == null) {
            content = text("loading…").dim();
        } else if (shells.isEmpty()) {
            content = text("no shells with scripts").dim();
        } else {
            boolean focused = st.focused == PanelId.SHELLS;
            content = Responsive.of(area -> Lists.selectable(shells, st.scripts.shellCursor, focused,
                    s -> shellRow(s, st, area.width())));
        }
        return Panels.framed(4, PanelId.SHELLS.elementId(), "Shells", content);
    }

    private static Row shellRow(ShellStatus status, AppState st, int innerWidth) {
        boolean primary = status.entry().id().equals(st.scripts.primaryShell);
        ShellIcon.Glyph icon = ShellIcon.of(status.detected(), status.deployed());
        String label = innerWidth < NARROW_WIDTH ? status.entry().id() : status.entry().name();
        return row(
                primary ? text("★ ").yellow() : text("  "),
                text(icon.symbol()).fg(icon.color()),
                text(" " + label));
    }
}
