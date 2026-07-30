package com.sampong.dotfile.ui.feature.search;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.ui.state.AppState;

import static dev.tamboui.toolkit.Toolkit.text;

/** MAIN/SEARCH_RESULTS: live package search results. Lands in Phase 7. */
public class SearchResultsView implements FeatureView {

    @Override
    public Element render(AppState st) {
        return text("Search results — Phase 7").dim();
    }
}
