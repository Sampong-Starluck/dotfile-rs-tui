package io.github.sampongstarluck.dotfile.service;

/** Port of {@code ../src/utils/text_util.rs}. */
public final class TextUtil {

    private TextUtil() {}

    /** Remove ANSI escape sequences (ESC[ ... letter, or ESC+1 char). */
    public static String stripAnsi(String s) {
        StringBuilder out = new StringBuilder(s.length());
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c == 0x1b) {
                if (i + 1 < s.length() && s.charAt(i + 1) == '[') {
                    i++; // consume '['
                    while (i + 1 < s.length()) {
                        i++;
                        char sc = s.charAt(i);
                        if (Character.isLetter(sc) && sc < 128) break;
                    }
                } else {
                    i++; // consume one char
                }
            } else {
                out.append(c);
            }
        }
        return out.toString();
    }

    /** Strip a trailing {@code \r} — needed on Windows piped output. */
    public static String sanitizeLine(String line) {
        return line.endsWith("\r") ? line.substring(0, line.length() - 1) : line;
    }

    /** char-index (NOT byte) of needle in haystack — for column offset detection. Returns -1 when absent. */
    public static int findCol(String haystack, String needle) {
        return haystack.indexOf(needle);
    }

    /** "neovim-0.9.5_1" -&gt; ["neovim", "0.9.5_1"]; version starts at the last '-' followed by a digit. */
    public static String[] splitPkgNameVersion(String pkg) {
        for (int i = pkg.length() - 1; i >= 1; i--) {
            if (pkg.charAt(i - 1) == '-' && Character.isDigit(pkg.charAt(i))) {
                return new String[] { pkg.substring(0, i - 1), pkg.substring(i) };
            }
        }
        return new String[] { pkg, "" };
    }
}
