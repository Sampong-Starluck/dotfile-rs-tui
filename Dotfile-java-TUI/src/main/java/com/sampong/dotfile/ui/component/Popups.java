package com.sampong.dotfile.ui.component;

import dev.tamboui.style.Color;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.DialogElement;

import java.util.List;

import static dev.tamboui.toolkit.Toolkit.dialog;
import static dev.tamboui.toolkit.Toolkit.text;

/**
 * Centered modal overlay. Built on the toolkit's {@code DialogElement}, which already
 * auto-centers and clears the background — no hand-rolled Rect math (PLAN.md Rule #1).
 */
public final class Popups {
    private Popups() {
    }

    public static DialogElement overlay(String title, Element... content) {
        return dialog(title, content).rounded().borderColor(Color.YELLOW);
    }

    /** Lazygit-style confirm dialog: max 10 items + "... and N more", y/n chip row. */
    public static DialogElement confirm(String title, List<String> items, String yesLabel, String noLabel) {
        boolean overflow = items.size() > 10;
        int shown = overflow ? 9 : items.size();
        Element[] lines = new Element[shown + (overflow ? 1 : 0) + 2];
        for (int i = 0; i < shown; i++) {
            lines[i] = text(items.get(i));
        }
        int next = shown;
        if (overflow) {
            lines[next++] = text("... and " + (items.size() - shown) + " more").dim();
        }
        lines[next++] = text("");
        lines[next] = text("[y] " + yesLabel + "   [n] " + noLabel).cyan();
        return dialog(title, lines).rounded().borderColor(Color.YELLOW);
    }
}
