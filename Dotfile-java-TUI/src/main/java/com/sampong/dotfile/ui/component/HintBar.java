package com.sampong.dotfile.ui.component;

import dev.tamboui.toolkit.elements.Row;

import java.util.List;

import static dev.tamboui.toolkit.Toolkit.row;
import static dev.tamboui.toolkit.Toolkit.text;

/** One dim line: {@code key: desc | key: desc ...}, keys cyan. */
public final class HintBar {
    private HintBar() {
    }

    public record Binding(String key, String desc) {
    }

    public static Row of(List<Binding> bindings) {
        Row line = row();
        for (int i = 0; i < bindings.size(); i++) {
            if (i > 0) {
                line.add(text(" │ ").dim());
            }
            Binding b = bindings.get(i);
            line.add(text(b.key()).cyan());
            line.add(text(": " + b.desc()).dim());
        }
        return line;
    }
}
