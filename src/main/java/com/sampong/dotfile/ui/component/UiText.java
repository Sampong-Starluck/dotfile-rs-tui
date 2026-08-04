package com.sampong.dotfile.ui.component;

/** Small text helpers not already covered by a built-in toolkit element. */
public final class UiText {
    private UiText() {
    }

    /** Code-point-aware truncate with a trailing ellipsis. */
    public static String truncate(String s, int max) {
        if (s == null) {
            return "";
        }
        int len = s.codePointCount(0, s.length());
        if (len <= max || max <= 0) {
            return s;
        }
        int cut = s.offsetByCodePoints(0, Math.max(0, max - 1));
        return s.substring(0, cut) + "…";
    }

    public static String padRight(String s, int width) {
        if (s == null) {
            s = "";
        }
        int len = s.codePointCount(0, s.length());
        if (len >= width) {
            return s;
        }
        return s + " ".repeat(width - len);
    }
}
