package com.sampong.dotfile.state;

import com.sampong.dotfile.model.SearchResult;
import org.jspecify.annotations.Nullable;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;

/** SEARCH_RESULTS main view (Phase 7). */
public class SearchState {
    public List<SearchResult> results = new ArrayList<>();
    public int cursor = 0;
    public boolean loading = false;
    public String lastQuery = "";
    public @Nullable CompletableFuture<List<SearchResult>> future = null;

    public void reset() {
        results = new ArrayList<>();
        cursor = 0;
        loading = false;
        lastQuery = "";
        future = null;
    }
}
