package com.sampong.dotfile.ui.component;

import dev.tamboui.style.Color;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Panel;

import static dev.tamboui.toolkit.Toolkit.panel;

/** Lazygit-styled framed panel(): numbered "[n]-Title" side panels, or a plain-titled main/popup panel. */
public final class Panels {
    private Panels() {
    }

    /** Numbered side panel, e.g. {@code [1]-Status}. */
    public static Panel framed(int number, String id, String title, Element... children) {
        return styled(panel("[" + number + "]─" + title, children), id);
    }

    /** Unnumbered panel (MAIN). */
    public static Panel framed(String id, String title, Element... children) {
        return styled(panel(title, children), id);
    }

    private static Panel styled(Panel p, String id) {
        return p.rounded()
                .id(id)
                .focusable()
                .borderColor(Color.DARK_GRAY)
                .focusedBorderColor(Color.GREEN);
    }
}
