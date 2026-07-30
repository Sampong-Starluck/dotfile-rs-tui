package com.sampong.dotfile.state;

import com.sampong.dotfile.model.AppSection;

import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

/** Sections panel + APPS main view. {@code apps} null means "(re)load on next render" (Phase 7). */
public class CatalogState {
    public List<AppSection> apps = null;
    public int sectionCursor = 0;
    public int appCursor = 0;
    public final Set<String> selectedIds = new LinkedHashSet<>();

    public void reset() {
        apps = null;
        sectionCursor = 0;
        appCursor = 0;
        selectedIds.clear();
    }
}
