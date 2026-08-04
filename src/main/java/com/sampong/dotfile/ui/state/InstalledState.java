package com.sampong.dotfile.ui.state;

import com.sampong.dotfile.model.SearchResult;
import com.sampong.dotfile.service.FuzzyMatcher;
import dev.tamboui.widgets.input.TextInputState;
import org.jspecify.annotations.Nullable;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
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
    public @Nullable CompletableFuture<List<SearchResult>> future = null;

    /** id -> available-version for packages with a pending update (Phase 13, net-new). Fetched
     *  alongside {@link #packages} by {@code CatalogActions.refreshInstalled}. */
    public Map<String, String> updates = new HashMap<>();
    public @Nullable CompletableFuture<Map<String, String>> updatesFuture = null;

    /** Client-side fuzzy filter over {@link #packages} (Phase 13, net-new — no CLI call
     *  involved, unlike {@code SearchInputPopup}'s remote search). {@code filtering} is whether
     *  the inline filter box is currently capturing keystrokes; the query itself stays applied
     *  (narrowing {@link #visiblePackages()}) even after the box is closed with Enter. */
    public final TextInputState filterQuery = new TextInputState();
    public boolean filtering = false;

    /** {@link #packages} narrowed by {@link #filterQuery}, matched fuzzily against name or id —
     *  what the view renders and what cursor/select/remove operate on, so acting on what's
     *  visible always matches what the user sees. */
    public List<SearchResult> visiblePackages() {
        String query = filterQuery.text();
        if (query.isEmpty()) {
            return packages;
        }
        return packages.stream()
                .filter(pkg -> FuzzyMatcher.matches(query, pkg.name()) || FuzzyMatcher.matches(query, pkg.id()))
                .toList();
    }

    public void reset() {
        packages = new ArrayList<>();
        names.clear();
        cursor = 0;
        loading = false;
        autoLoaded = false;
        removeMode = false;
        future = null;
        updates = new HashMap<>();
        updatesFuture = null;
        filterQuery.clear();
        filtering = false;
    }
}
