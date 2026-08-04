package com.sampong.dotfile.ui.component;

import dev.tamboui.toolkit.elements.Column;
import dev.tamboui.toolkit.elements.TextElement;

import java.util.List;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.text;

/** Colored, autoscrolled log column: last {@code maxRows} lines, colored by leading glyph. */
public final class Logs {
    private Logs() {
    }

    public static Column colored(List<String> lines, int maxRows) {
        Column col = column();
        int from = Math.max(0, lines.size() - maxRows);
        for (int i = from; i < lines.size(); i++) {
            col.add(colorize(lines.get(i)));
        }
        return col;
    }

    private static TextElement colorize(String line) {
        TextElement t = text(line);
        if (line.startsWith("✓")) {
            return t.green();
        }
        if (line.startsWith("✗") || line.contains("[err]")) {
            return t.red();
        }
        if (line.startsWith("▶")) {
            return t.cyan();
        }
        if (line.startsWith("═")) {
            return t.yellow().bold();
        }
        if (line.startsWith("★")) {
            return t.yellow();
        }
        return t.dim();
    }
}
