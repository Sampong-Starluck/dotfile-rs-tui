package com.sampong.dotfile.ui.component;

import dev.tamboui.style.Color;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.element.RenderContext;
import dev.tamboui.toolkit.elements.DialogElement;

import java.util.List;

import static dev.tamboui.toolkit.Toolkit.dialog;
import static dev.tamboui.toolkit.Toolkit.text;

/**
 * Centered modal overlay. Built on the toolkit's {@code DialogElement}, which already
 * auto-centers and clears the background — no hand-rolled Rect math (PLAN.md Rule #1).
 * <p>
 * {@code DialogElement}'s own auto-width has a bug: the width it actually renders at
 * ({@code calculateWidth}) ignores its children completely (title length vs. a 20-col
 * floor only), while the width it *reports* to callers ({@code preferredSize}) correctly
 * measures the widest child — but nothing ever reads that value, since the dialog's
 * {@code constraint()} unconditionally fills its parent. Found via human review: the
 * search popup's hint line was clipped mid-word. {@link #sized} pins {@code fixedWidth}
 * from the dialog's own (correct) {@code preferredSize()}, which bypasses the broken path.
 */
public final class Popups {
    private Popups() {
    }

    public static DialogElement overlay(String title, Element... content) {
        return sized(dialog(title, content).rounded().borderColor(Color.YELLOW));
    }

    /** Lazygit-style confirm dialog: warning line, max 10 items + "... and N more", y/n chip row. */
    public static DialogElement confirm(String title, String warning, List<String> items, String yesLabel, String noLabel) {
        boolean overflow = items.size() > 10;
        int shown = overflow ? 9 : items.size();
        Element[] lines = new Element[1 + shown + (overflow ? 1 : 0) + 2];
        lines[0] = text("⚠ " + warning).yellow();
        for (int i = 0; i < shown; i++) {
            lines[i + 1] = text(items.get(i));
        }
        int next = 1 + shown;
        if (overflow) {
            lines[next++] = text("... and " + (items.size() - shown) + " more").dim();
        }
        lines[next++] = text("");
        lines[next] = text("[y] " + yesLabel + "   [n] " + noLabel).cyan();
        return sized(dialog(title, lines).rounded().borderColor(Color.YELLOW));
    }

    private static DialogElement sized(DialogElement dialog) {
        int width = dialog.preferredSize(-1, -1, RenderContext.empty()).width();
        return dialog.width(width);
    }
}
