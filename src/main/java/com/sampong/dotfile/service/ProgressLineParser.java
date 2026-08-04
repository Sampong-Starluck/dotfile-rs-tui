package com.sampong.dotfile.service;

import org.jspecify.annotations.Nullable;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Extracts download/install progress numbers out of the same winget stdout lines
 * {@link DecodeUtil#isNoiseLine} already classifies as spinner/progress junk (PLAN.md
 * phase-12 §12.1) — this never reads winget's own bar glyphs (only ever observed as a solid
 * "{@code █}" run with no distinguishable partial-fill character), only the numeric text next
 * to them.
 */
public final class ProgressLineParser {

    private ProgressLineParser() {
    }

    private static final Pattern SIZE = Pattern.compile(
            "([0-9]+(?:\\.[0-9]+)?)\\s*(B|KB|MB|GB)\\s*/\\s*([0-9]+(?:\\.[0-9]+)?)\\s*(B|KB|MB|GB)");
    private static final Pattern PERCENT = Pattern.compile("([0-9]{1,3})%");

    /** {@code downloadedText}/{@code totalText} are null for a percent-only (install-phase) line. */
    public record Progress(@Nullable String downloadedText, @Nullable String totalText, int percent) {
    }

    public static Optional<Progress> parse(String line) {
        Matcher size = SIZE.matcher(line);
        if (size.find()) {
            double downloaded = Double.parseDouble(size.group(1)) * unitBytes(size.group(2));
            double total = Double.parseDouble(size.group(3)) * unitBytes(size.group(4));
            int percent = total <= 0 ? 0 : (int) Math.round(downloaded / total * 100);
            return Optional.of(new Progress(
                    size.group(1) + " " + size.group(2), size.group(3) + " " + size.group(4), percent));
        }
        Matcher percent = PERCENT.matcher(line);
        if (percent.find()) {
            return Optional.of(new Progress(null, null, Integer.parseInt(percent.group(1))));
        }
        return Optional.empty();
    }

    private static double unitBytes(String unit) {
        return switch (unit) {
            case "KB" -> 1024.0;
            case "MB" -> 1024.0 * 1024;
            case "GB" -> 1024.0 * 1024 * 1024;
            default -> 1.0;
        };
    }
}
