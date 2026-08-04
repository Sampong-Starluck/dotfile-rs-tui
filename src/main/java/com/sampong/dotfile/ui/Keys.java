package com.sampong.dotfile.ui;

import dev.tamboui.tui.event.KeyCode;
import dev.tamboui.tui.event.KeyEvent;

/** Static predicates over the real {@link KeyEvent}. Navigation predicates are for list controllers only. */
public final class Keys {
    private Keys() {
    }

    public static boolean isUp(KeyEvent e) {
        return e.code() == KeyCode.UP || e.isChar('k');
    }

    public static boolean isDown(KeyEvent e) {
        return e.code() == KeyCode.DOWN || e.isChar('j');
    }

    public static boolean isEnter(KeyEvent e) {
        return e.code() == KeyCode.ENTER;
    }

    public static boolean isEsc(KeyEvent e) {
        return e.code() == KeyCode.ESCAPE;
    }

    public static boolean isPageUp(KeyEvent e) {
        return e.code() == KeyCode.PAGE_UP;
    }

    public static boolean isPageDown(KeyEvent e) {
        return e.code() == KeyCode.PAGE_DOWN;
    }

    public static boolean isBackspace(KeyEvent e) {
        return e.code() == KeyCode.BACKSPACE;
    }

    public static boolean isChar(KeyEvent e, char c) {
        return e.isChar(c);
    }

    /** Printable code point, or -1 for non-CHAR events. */
    public static int charOf(KeyEvent e) {
        return e.code() == KeyCode.CHAR ? e.codePoint() : -1;
    }

    /** 1-4 for digit keys '1'...'4', else -1. */
    public static int digitOf(KeyEvent e) {
        int cp = charOf(e);
        return (cp >= '1' && cp <= '4') ? (cp - '0') : -1;
    }
}
