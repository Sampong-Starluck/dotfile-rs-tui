package com.sampong.dotfile.ui.layout;

import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Column;
import com.sampong.dotfile.ui.component.Responsive;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.row;

/**
 * Composes the lazygit frame (PLAN.md 5) as one fluent tree.
 * <p>
 * The side column's {@code min(24)/max(42)} clamp (PLAN.md 5, widened from the original
 * max(34) after human review found panel content — e.g. package-manager descriptions —
 * routinely got clipped at 34 cols) needs the real terminal width, which isn't known
 * while the tree is being built — Cassowary's {@code Min}
 * constraint alone competes for leftover space like {@code Fill} and has no upper
 * bound, so on a wide terminal it split roughly 50/50 with the main panel instead of
 * staying near a third. {@code body} is wrapped in {@link Responsive} so the clamp is
 * computed from the real area once it's known, at render time.
 */
public final class LazygitLayout {
    private LazygitLayout() {
    }

    public static Column frame(Element status, Element managers, Element sections,
                                Element shells, Element main, Element hints,
                                int pmCount, int shellCount) {
        Element body = Responsive.of(area -> {
            Column side = column(
                    column(status).length(3),
                    column(managers).length(clamp(pmCount, 1, 6) + 2),
                    column(sections).fill(),
                    column(shells).length(clamp(shellCount, 1, 7) + 2)
            ).length(clamp(area.width() / 3, 24, 42));

            return row(side, column(main).fill());
        });

        return column(body, column(hints).length(1));
    }

    private static int clamp(int value, int min, int max) {
        return Math.min(Math.max(value, min), max);
    }
}
