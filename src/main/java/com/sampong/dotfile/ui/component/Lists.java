package com.sampong.dotfile.ui.component;

import dev.tamboui.style.Color;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.element.StyledElement;
import dev.tamboui.toolkit.elements.Column;

import java.util.List;
import java.util.function.Function;

import static dev.tamboui.toolkit.Toolkit.column;

/**
 * Selectable list: rows + cursor + selection markers.
 * <p>
 * The cursor lives in {@code state/} (e.g. {@code platform.managersCursor}), not in the widget
 * — this composes a plain {@link Column} of styled rows rather than the toolkit's stateful
 * {@code ListElement}, so there is exactly one source of truth for selection (PLAN.md 5a).
 * <p>
 * Wrapped in {@link Responsive} so the visible window can follow the cursor: without it, a list
 * taller than its panel just gets clipped by the layout, and once the cursor scrolls past the
 * clipped edge the highlight appears to vanish entirely (found via human review of Phase 9's real
 * {@code winget list} output — up to 100+ rows, vs. the short/stub lists earlier phases exercised).
 * The window is derived fresh from {@code cursor} every frame rather than persisted per-list
 * scroll state (unlike {@code PlatformState.commandScroll}, which is user-driven): it always
 * pins the cursor to the last visible row once scrolled, which is simpler and needs no new state.
 */
public final class Lists {
    private Lists() {
    }

    public static <T, E extends StyledElement<E>> Element selectable(
            List<T> items, int cursor, boolean focused, Function<T, E> row) {
        return Responsive.of(area -> {
            int visibleRows = Math.max(1, area.height());
            int maxScroll = Math.max(0, items.size() - visibleRows);
            int scroll = Math.max(0, Math.min(cursor - visibleRows + 1, maxScroll));
            int end = Math.min(items.size(), scroll + visibleRows);

            Column col = column();
            for (int i = scroll; i < end; i++) {
                E el = row.apply(items.get(i));
                if (i == cursor) {
                    el = focused ? el.reversed().bold() : el.bold().fg(Color.CYAN);
                }
                col.add(el);
            }
            return col;
        });
    }
}
