package com.sampong.dotfile.service;

import java.util.Locale;

/**
 * Net-new (Phase 13 — no Rust prior art, same as the phase-12 progress bar): simple
 * subsequence fuzzy match for narrowing the installed-packages list as the user types. A query
 * matches when every one of its characters appears in the candidate, in order, case-insensitive
 * — not necessarily contiguous (e.g. "dbv" matches "DBeaver").
 */
public final class FuzzyMatcher {

    private FuzzyMatcher() {
    }

    public static boolean matches(String query, String candidate) {
        if (query.isEmpty()) {
            return true;
        }
        String q = query.toLowerCase(Locale.ROOT);
        String c = candidate.toLowerCase(Locale.ROOT);
        int qi = 0;
        for (int ci = 0; ci < c.length() && qi < q.length(); ci++) {
            if (c.charAt(ci) == q.charAt(qi)) {
                qi++;
            }
        }
        return qi == q.length();
    }
}
