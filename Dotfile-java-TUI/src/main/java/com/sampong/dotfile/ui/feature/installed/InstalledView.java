package com.sampong.dotfile.ui.feature.installed;

import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Row;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.model.SearchResult;
import com.sampong.dotfile.ui.component.Lists;
import com.sampong.dotfile.ui.component.Responsive;
import com.sampong.dotfile.ui.component.UiText;
import com.sampong.dotfile.ui.state.AppState;

import java.util.List;

import static dev.tamboui.toolkit.Toolkit.row;
import static dev.tamboui.toolkit.Toolkit.spinner;
import static dev.tamboui.toolkit.Toolkit.text;

/** MAIN/INSTALLED: port of {@code app_tab.rs::render_installed_panel} — picked rows read red (remove semantics). */
public class InstalledView implements FeatureView {

    private static final int VERSION_WIDTH = 16;

    @Override
    public Element render(AppState st) {
        if (st.installed.loading) {
            return row(spinner(), text(" loading…").yellow());
        }
        List<SearchResult> packages = st.installed.packages;
        if (packages.isEmpty()) {
            return text("no installed packages — r: refresh").dim();
        }
        boolean focused = st.focused == PanelId.MAIN;
        return Responsive.of(area -> {
            int nameW = Math.max(area.width() - VERSION_WIDTH - 8, 0);
            return Lists.selectable(packages, st.installed.cursor, focused,
                    pkg -> packageRow(pkg, st, nameW));
        });
    }

    @Override
    public String title(AppState st) {
        return "Installed " + st.platform.activeBinary() + " — " + st.installed.packages.size() + " package(s)";
    }

    private static Row packageRow(SearchResult pkg, AppState st, int nameW) {
        boolean picked = st.catalog.selectedIds.contains(pkg.id());
        String checkbox = picked ? "[✓]" : "[ ]";
        String name = UiText.padRight(UiText.truncate(pkg.name(), nameW), nameW) + "  ";
        String version = UiText.padRight(UiText.truncate(pkg.version(), VERSION_WIDTH), VERSION_WIDTH);

        Row line = row(text(checkbox + " "));
        if (picked) {
            line.add(text(name).red().bold());
            line.add(text(version).red());
        } else {
            line.add(text(name));
            line.add(text(version).dim());
        }
        return line;
    }
}
