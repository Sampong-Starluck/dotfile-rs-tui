package com.sampong.dotfile.ui.feature.search;

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

/** MAIN/SEARCH_RESULTS: port of {@code app_tab.rs::render_search_panel} styling. */
public class SearchResultsView implements FeatureView {

    @Override
    public Element render(AppState st) {
        if (st.search.loading) {
            return row(spinner(), text(" searching…").yellow());
        }
        List<SearchResult> results = st.search.results;
        if (results.isEmpty()) {
            return text(st.search.lastQuery.isEmpty()
                    ? "Type / to search."
                    : "No results found — try a different query.").dim();
        }
        boolean focused = st.focused == PanelId.MAIN;
        return Responsive.of(area -> {
            int available = Math.max(area.width() - 6 - 4, 0);
            int idW = (int) (available * 0.40);
            int nameW = (int) (available * 0.40);
            int verW = Math.max(available - idW - nameW, 0);
            return Lists.selectable(results, st.search.cursor, focused,
                    r -> resultRow(r, st, idW, nameW, verW));
        });
    }

    @Override
    public String title(AppState st) {
        return "Search " + st.platform.activeBinary() + " — " + st.search.results.size() + " result(s)";
    }

    private static Row resultRow(SearchResult r, AppState st, int idW, int nameW, int verW) {
        boolean picked = st.catalog.selectedIds.contains(r.id());
        boolean installed = st.installed.names.contains(r.name()) || st.installed.names.contains(r.id());
        String checkbox = picked ? "[✓]" : installed ? "[I]" : "[ ]";

        return row(
                text(checkbox + " "),
                text(UiText.padRight(UiText.truncate(r.id(), idW), idW) + "  ").yellow(),
                text(UiText.padRight(UiText.truncate(r.name(), nameW), nameW) + "  "),
                text(UiText.padRight(UiText.truncate(r.version(), verW), verW)).dim());
    }
}
