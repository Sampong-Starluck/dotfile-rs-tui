package com.sampong.dotfile.state;

import com.sampong.dotfile.model.SearchResult;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.CompletableFuture;

/** INSTALLED main view (Phase 7). Auto-loaded once in the background, never on the startup path. */
public class InstalledState {
    public List<SearchResult> packages = new ArrayList<>();
    public final Set<String> names = new HashSet<>();
    public int cursor = 0;
    public boolean loading = false;
    public boolean autoLoaded = false;
    public boolean removeMode = false;
    public CompletableFuture<List<SearchResult>> future = null;

    public void reset() {
        packages = new ArrayList<>();
        names.clear();
        cursor = 0;
        loading = false;
        autoLoaded = false;
        removeMode = false;
        future = null;
    }
}
