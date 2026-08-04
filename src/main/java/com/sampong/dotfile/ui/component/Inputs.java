package com.sampong.dotfile.ui.component;

import dev.tamboui.toolkit.elements.TextInputElement;
import dev.tamboui.widgets.input.TextInputState;

import static dev.tamboui.toolkit.Toolkit.textInput;

/** Single-line input: wraps the toolkit's real text-input element + state (cursor/editing built in). */
public final class Inputs {
    private Inputs() {
    }

    public static TextInputElement line(TextInputState state, String placeholder) {
        return textInput(state).placeholder(placeholder);
    }
}
