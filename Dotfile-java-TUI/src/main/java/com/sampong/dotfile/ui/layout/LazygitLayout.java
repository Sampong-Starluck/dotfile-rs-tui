package com.sampong.dotfile.ui.layout;

import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Column;
import com.sampong.dotfile.ui.component.Responsive;
import com.sampong.dotfile.ui.component.Sized;
import com.sampong.dotfile.ui.feature.managers.ManagersView;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.row;

/**
 * Composes the lazygit frame (PLAN.md 5) as one fluent tree.
 * <p>
 * The side column's {@code min(24)/max(42)} clamp (PLAN.md 5, widened from the original
 * max(34) after human review found panel content — e.g. package-manager descriptions —
 * routinely got clipped at 34 cols) needs the real terminal width, which isn't known while
 * the tree is being built — Cassowary's {@code Min} constraint alone competes for leftover
 * space like {@code Fill} and has no upper bound, so on a wide terminal it split roughly
 * 50/50 with the main panel instead of staying near a third. {@code body} is wrapped in
 * {@link Responsive} so the clamp is computed from the real area once it's known, at render
 * time. {@link ManagersView#panelHeight} similarly reserves 2 lines per manager row now that
 * descriptions wrap onto their own line instead of being truncated inline.
 * <p>
 * Each panel slot is wrapped with {@link Sized}, not a plain {@code column(panel).fill()}/
 * {@code .length(n)}: a plain column only reports that constraint to ITS OWN parent, then
 * independently re-derives a constraint for its single child from the child's
 * {@code preferredSize()} — which for a bordered panel is always a real content-sized number,
 * never "unknown" — so the panel got pinned to its content height and the rest of the slot's
 * allocated area rendered blank (found via human review: panels collapsed to content size
 * instead of filling their slot, worst on {@code Sections}/{@code MAIN}). {@link Sized} exposes
 * the constraint directly and pass-through renders the whole slot area to the panel, avoiding
 * that re-derivation.
 */
public final class LazygitLayout {
    private LazygitLayout() {
    }

    public static Column frame(Element status, Element managers, Element sections,
                                Element shells, Element main, Element hints,
                                int pmCount, int shellCount) {
        Element body = Responsive.of(area -> {
            Column side = column(
                    Sized.length(status, 3),
                    Sized.length(managers, ManagersView.panelHeight(pmCount)),
                    Sized.fill(sections),
                    Sized.length(shells, clamp(shellCount, 1, 7) + 2)
            ).length(clamp(area.width() / 3, 24, 42));

            return row(side, Sized.fill(main));
        });

        return column(body, Sized.length(hints, 1));
    }

    private static int clamp(int value, int min, int max) {
        return Math.min(Math.max(value, min), max);
    }
}
