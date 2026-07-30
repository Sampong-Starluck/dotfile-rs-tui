package com.sampong.dotfile.ui.layout;

import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Column;
import dev.tamboui.toolkit.elements.Row;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.row;

/**
 * Composes the lazygit frame (PLAN.md 5) as one fluent tree — no Rect math.
 * <p>
 * Deviation: the side column uses a fixed {@code .min(24)} floor rather than a true
 * {@code min(24)/max(34)} clamp — the Cassowary {@code Constraint} model has no
 * composite min+max for a single slot, and computing the clamp from the terminal
 * width would mean the Rect math Rule #1 tells us to avoid. See FEATURE-PARITY.md.
 */
public final class LazygitLayout {
    private LazygitLayout() {
    }

    public static Column frame(Element status, Element managers, Element sections,
                                Element shells, Element main, Element hints,
                                int pmCount, int shellCount) {
        Column side = column(
                column(status).length(3),
                column(managers).length(clamp(pmCount, 1, 6) + 2),
                column(sections).fill(),
                column(shells).length(clamp(shellCount, 1, 7) + 2)
        ).min(24);

        Row body = row(side, column(main).fill());

        return column(body.fill(), column(hints).length(1));
    }

    private static int clamp(int value, int min, int max) {
        return Math.min(Math.max(value, min), max);
    }
}
