package com.sampong.dotfile.ui.feature.installed;

import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Row;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.model.SearchResult;
import com.sampong.dotfile.ui.component.Inputs;
import com.sampong.dotfile.ui.component.Lists;
import com.sampong.dotfile.ui.component.Responsive;
import com.sampong.dotfile.ui.component.Sized;
import com.sampong.dotfile.ui.component.UiText;
import com.sampong.dotfile.ui.state.AppState;

import java.util.List;
import java.util.function.Function;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.row;
import static dev.tamboui.toolkit.Toolkit.spinner;
import static dev.tamboui.toolkit.Toolkit.text;

/** MAIN/INSTALLED: port of {@code app_tab.rs::render_installed_panel} — picked rows read red
 *  (remove semantics). Phase 13 (net-new, no Rust prior art) adds a "New Version" column
 *  (populated from {@code st.installed.updates}, an id -> available-version map fetched
 *  alongside the installed list) and an inline fuzzy-filter box. */
public class InstalledView implements FeatureView {

    /** Version columns size to the longest actual value currently on screen (some managers'
     *  version strings run long, e.g. yt-dlp's FFmpeg build "N-123778-g3b55818764-20260331"),
     *  clamped to this range rather than a fixed width that would truncate them. */
    private static final int MIN_VERSION_WIDTH = 10;
    private static final int MAX_VERSION_WIDTH = 40;

    @Override
    public Element render(AppState st) {
        if (st.installed.loading) {
            return row(spinner(), text(" loading…").yellow());
        }
        if (st.installed.packages.isEmpty()) {
            return text("no installed packages — r: refresh").dim();
        }
        boolean focused = st.focused == PanelId.MAIN;
        List<SearchResult> visible = st.installed.visiblePackages();
        boolean showFilterRow = st.installed.filtering || !st.installed.filterQuery.text().isEmpty();
        int versionW = columnWidth(visible, SearchResult::version);
        int newVersionW = columnWidth(visible, pkg -> st.installed.updates.getOrDefault(pkg.id(), ""));

        Element list = Responsive.of(area -> {
            int nameW = Math.max(area.width() - versionW - newVersionW - 12, 0);
            if (visible.isEmpty()) {
                return text("no matches for \"" + st.installed.filterQuery.text() + "\"").dim();
            }
            return Lists.selectable(visible, st.installed.cursor, focused,
                    pkg -> packageRow(pkg, st, nameW, versionW, newVersionW));
        });

        if (!showFilterRow) {
            return list;
        }
        return Responsive.of(area -> column(
                Sized.length(filterRow(st), 1),
                Sized.fill(list)));
    }

    @Override
    public String title(AppState st) {
        List<SearchResult> visible = st.installed.visiblePackages();
        StringBuilder title = new StringBuilder("Installed ").append(st.platform.activeBinary()).append(" — ");
        if (visible.size() != st.installed.packages.size()) {
            title.append(visible.size()).append('/').append(st.installed.packages.size()).append(" package(s)");
        } else {
            title.append(st.installed.packages.size()).append(" package(s)");
        }
        if (!st.installed.updates.isEmpty()) {
            title.append(", ").append(st.installed.updates.size()).append(" update(s)");
        }
        return title.toString();
    }

    private static Row filterRow(AppState st) {
        return row(text("⌕ ").cyan(),
                Inputs.line(st.installed.filterQuery, "filter…").focusable(false).cursorRequiresFocus(false));
    }

    private static Row packageRow(SearchResult pkg, AppState st, int nameW, int versionW, int newVersionW) {
        boolean picked = st.catalog.selectedIds.contains(pkg.id());
        String checkbox = picked ? "[✓]" : "[ ]";
        String name = UiText.padRight(UiText.truncate(pkg.name(), nameW), nameW) + "  ";
        String version = UiText.padRight(UiText.truncate(pkg.version(), versionW), versionW);
        String newVersion = st.installed.updates.getOrDefault(pkg.id(), "");
        String newVersionText = "  " + UiText.padRight(UiText.truncate(newVersion, newVersionW), newVersionW);

        Row line = row(text(checkbox + " "));
        if (picked) {
            line.add(text(name).red().bold());
            line.add(text(version).red());
        } else {
            line.add(text(name));
            line.add(text(version).dim());
        }
        line.add(newVersion.isEmpty() ? text(newVersionText) : text(newVersionText).green());
        return line;
    }

    private static int columnWidth(List<SearchResult> items, Function<SearchResult, String> field) {
        int max = MIN_VERSION_WIDTH;
        for (SearchResult item : items) {
            String value = field.apply(item);
            max = Math.max(max, value.codePointCount(0, value.length()));
        }
        return Math.min(MAX_VERSION_WIDTH, max);
    }
}
