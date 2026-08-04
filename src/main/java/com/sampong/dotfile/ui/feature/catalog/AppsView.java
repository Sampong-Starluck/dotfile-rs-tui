package com.sampong.dotfile.ui.feature.catalog;

import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Row;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.AppEntry;
import com.sampong.dotfile.model.AppSection;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.ui.component.Lists;
import com.sampong.dotfile.ui.state.AppState;

import java.util.List;
import java.util.Map;
import java.util.Set;

import static dev.tamboui.toolkit.Toolkit.row;
import static dev.tamboui.toolkit.Toolkit.text;

/** MAIN/APPS: apps of the highlighted section — port of {@code app_tab.rs::render_app_list}. */
public class AppsView implements FeatureView {

    @Override
    public Element render(AppState st) {
        AppSection section = currentSection(st);
        if (section == null) {
            return text("No section selected").dim();
        }
        String mgr = st.platform.activeBinary();
        boolean focused = st.focused == PanelId.MAIN;

        return Lists.selectable(section.apps(), st.catalog.appCursor, focused,
                entry -> appRow(entry, st, mgr));
    }

    @Override
    public String title(AppState st) {
        AppSection section = currentSection(st);
        if (section == null) {
            return "Apps";
        }
        int n = st.catalog.selectedIds.size();
        String badge = n > 0 ? "  ✓ " + n : "";
        return section.section() + " — " + st.platform.activeBinary() + badge;
    }

    private static AppSection currentSection(AppState st) {
        List<AppSection> apps = st.catalog.apps;
        if (apps == null || apps.isEmpty() || st.catalog.sectionCursor >= apps.size()) {
            return null;
        }
        return apps.get(st.catalog.sectionCursor);
    }

    private static Row appRow(AppEntry entry, AppState st, String mgr) {
        boolean selected = st.catalog.selectedIds.contains(entry.id());
        boolean installed = isInstalled(entry, st, mgr);
        String checkbox = selected ? "[✓]" : installed ? "[I]" : "[ ]";

        Row line = row(text(checkbox + " "));
        if (selected) {
            line.add(text(entry.name()).green().bold());
        } else if (installed) {
            line.add(text(entry.name()).green());
        } else {
            line.add(text(entry.name()));
        }
        return line;
    }

    /** Port of {@code app_tab.rs::entry_installed}: resolved package name in the installed set, id as fallback. */
    private static boolean isInstalled(AppEntry entry, AppState st, String mgr) {
        String osKey = st.platform.os != null ? st.platform.os.key() : "linux";
        Map<String, String> platform = entry.platforms().get(osKey);
        Set<String> installedNames = st.installed.names;
        String pkg = platform != null ? platform.get(mgr) : null;
        return pkg != null ? installedNames.contains(pkg) : installedNames.contains(entry.id());
    }
}
