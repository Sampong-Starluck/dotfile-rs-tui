package com.sampong.dotfile.ui.component;

import dev.tamboui.toolkit.elements.Column;
import org.jspecify.annotations.Nullable;

import java.util.Arrays;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.text;

/**
 * Arrow-style download/install progress bar (PLAN.md phase-12 §12.1), e.g.
 * {@code [======>>    ]  45%  12.1 MB / 38.6 MB}. Renders from a parsed percentage, never from
 * winget's own bar glyphs — see {@code service.ProgressLineParser}'s javadoc for why.
 */
public final class ProgressBar {
    private ProgressBar() {
    }

    private static final int WIDTH = 30;

    public static Column render(@Nullable String label, @Nullable String downloadedText,
                                 @Nullable String totalText, int percent) {
        Column col = column();
        if (label != null) {
            col.add(text(label).dim());
        }
        String sizeSuffix = (downloadedText != null && totalText != null)
                ? "  " + downloadedText + " / " + totalText
                : "";
        col.add(text(bar(percent) + "  " + clamp(percent) + "%" + sizeSuffix).cyan());
        return col;
    }

    private static String bar(int percent) {
        int filled = WIDTH * clamp(percent) / 100;
        char[] chars = new char[WIDTH];
        Arrays.fill(chars, ' ');
        for (int i = 0; i < filled; i++) {
            chars[i] = '=';
        }
        if (filled > 0) {
            chars[filled - 1] = '>';
        }
        if (filled > 1) {
            chars[filled - 2] = '>';
        }
        return "[" + new String(chars) + "]";
    }

    private static int clamp(int percent) {
        return Math.max(0, Math.min(100, percent));
    }
}
