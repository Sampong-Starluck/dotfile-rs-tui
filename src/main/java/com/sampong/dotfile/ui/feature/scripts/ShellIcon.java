package com.sampong.dotfile.ui.feature.scripts;

import dev.tamboui.style.Color;

/** Port of Rust {@code script_tab.rs::shell_icon}, shared by {@link ShellsView}'s row glyph and
 *  {@link ShellInfoView}'s status line (which also needs the label Rust inlines separately). */
final class ShellIcon {
    private ShellIcon() {
    }

    record Glyph(String symbol, Color color, String label) {
    }

    static Glyph of(boolean detected, boolean deployed) {
        if (!detected) {
            return new Glyph("○", Color.DARK_GRAY, "not installed on this system");
        }
        if (deployed) {
            return new Glyph("✓", Color.GREEN, "deployed");
        }
        return new Glyph("◆", Color.YELLOW, "detected — not deployed");
    }
}
