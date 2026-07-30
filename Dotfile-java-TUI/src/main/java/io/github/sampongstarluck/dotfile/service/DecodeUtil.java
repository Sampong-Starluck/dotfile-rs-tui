package io.github.sampongstarluck.dotfile.service;

import java.nio.charset.StandardCharsets;

/** Port of {@code ../src/utils/decode_util.rs}. */
public final class DecodeUtil {

    private DecodeUtil() {}

    /** winget may emit UTF-16 LE with BOM; also handle UTF-8 BOM; else UTF-8. */
    public static String decodeWingetOutput(byte[] bytes) {
        if (bytes.length >= 2 && (bytes[0] & 0xFF) == 0xFF && (bytes[1] & 0xFF) == 0xFE) {
            return new String(bytes, 2, bytes.length - 2, StandardCharsets.UTF_16LE);
        }
        if (bytes.length >= 3 && (bytes[0] & 0xFF) == 0xEF && (bytes[1] & 0xFF) == 0xBB
                && (bytes[2] & 0xFF) == 0xBF) {
            return new String(bytes, 3, bytes.length - 3, StandardCharsets.UTF_8);
        }
        return new String(bytes, StandardCharsets.UTF_8);
    }

    /** Port of {@code is_noise_line()}: filter winget spinner/progress junk from install-log output. */
    public static boolean isNoiseLine(String s) {
        String t = s.trim();
        if (t.isEmpty()) return true;
        if (t.equals("-") || t.equals("\\") || t.equals("|") || t.equals("/")) return true;
        if (t.contains("█") || t.contains("▒")) return true;
        if (t.contains("KB /") || t.contains("MB /") || t.contains("GB /")) return true;
        if (t.endsWith("%")) {
            String num = t.substring(0, t.length() - 1).trim();
            if (!num.isEmpty() && num.chars().allMatch(Character::isDigit)) return true;
        }
        return false;
    }
}
