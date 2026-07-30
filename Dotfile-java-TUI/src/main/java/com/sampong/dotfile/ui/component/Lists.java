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
 */
public final class Lists {
    private Lists() {
    }

    public static <T, E extends StyledElement<E>> Element selectable(
            List<T> items, int cursor, boolean focused, Function<T, E> row) {
        Column col = column();
        for (int i = 0; i < items.size(); i++) {
            E el = row.apply(items.get(i));
            if (i == cursor) {
                el = focused ? el.reversed().bold() : el.bold().fg(Color.CYAN);
            }
            col.add(el);
        }
        return col;
    }
}
