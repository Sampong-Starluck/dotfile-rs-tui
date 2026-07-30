package com.sampong.dotfile.service;

import com.sampong.dotfile.model.SearchResult;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/**
 * Port of {@code ../src/service/search_service.rs} decoding/parsing (command tables
 * live in {@link SearchService}/{@code SearchServiceImp} from Phase 3). Pure static
 * functions — no I/O, no Spring, no TamboUI.
 */
public final class OutputParsers {

    private OutputParsers() {}

    public static String decodeSearchOutput(String mgr, byte[] bytes) {
        return mgr.equals("winget") ? DecodeUtil.decodeWingetOutput(bytes)
                                    : new String(bytes, StandardCharsets.UTF_8);
    }

    public static List<SearchResult> parseSearchOutput(String mgr, String text) {
        return switch (mgr) {
            case "scoop"                 -> parseScoopSearch(text);
            case "choco"                 -> parseChocoSearch(text);
            case "pacman", "yay", "paru" -> parsePacmanSearch(text);
            case "apt", "apt-get"        -> parseAptSearch(text);
            case "dnf", "yum"            -> parseDnfSearch(text);
            case "xbps-install"          -> parseXbpsSearch(text);
            case "apk"                   -> parseApkSearch(text);
            default                      -> parseWingetSearch(text);
        };
    }

    public static List<SearchResult> parseListOutput(String mgr, String text) {
        return switch (mgr) {
            case "pacman", "yay", "paru" -> parsePacmanList(text);
            case "apt", "apt-get"        -> parseAptList(text);
            case "dnf", "yum"            -> parseDnfList(text);
            case "xbps-install"          -> parseXbpsList(text);
            case "apk"                   -> parseApkList(text);
            case "brew"                  -> parseBrewList(text);
            case "winget"                -> parseWingetSearch(text);   // same tabular format
            case "scoop"                 -> parseScoopSearch(text);
            case "choco"                 -> parseChocoList(text);
            default                      -> List.of();
        };
    }

    // ─── Search parsers ────────────────────────────────────────────────────

    /**
     * Parse {@code winget search} tabular output. Winget animates progress with bare
     * {@code \r}, so the whole output is split on both {@code \r} and {@code \n}
     * (not treated as a combined line terminator) before ANSI stripping and column
     * detection. Data-row slicing is code-point based (winget rows can contain
     * non-BMP characters and the header column offsets are in chars, not UTF-16 units).
     */
    private static List<SearchResult> parseWingetSearch(String output) {
        List<SearchResult> results = new ArrayList<>();
        int[] headerOffsets = null; // {idStart, verStart, srcStart}

        for (String segment : output.split("[\r\n]")) {
            String line = TextUtil.stripAnsi(segment);

            if (line.trim().isEmpty()) continue;
            if (line.contains("█") || line.contains("▒") || line.contains("KB /") || line.contains("MB /")) continue;
            String trimmed = line.trim();
            if (trimmed.equals("-") || trimmed.equals("\\") || trimmed.equals("|") || trimmed.equals("/")) continue;

            if (headerOffsets == null) {
                String lower = line.toLowerCase(Locale.ROOT);
                if (lower.contains("name") && lower.contains("id") && lower.contains("version")) {
                    int idPos = orElseFindCol(line, "Id", "id");
                    int verPos = orElseFindCol(line, "Version", "version");
                    int srcPos = orElseFindCol(line, "Source", "source");
                    if (idPos > 0 && verPos > idPos) {
                        headerOffsets = new int[] { idPos, verPos, srcPos >= 0 ? srcPos : Integer.MAX_VALUE };
                    }
                }
                continue;
            }

            if (isDashSeparatorLine(line)) continue;

            int idStart = headerOffsets[0];
            int verStart = headerOffsets[1];
            int srcStart = headerOffsets[2];
            int[] cps = line.codePoints().toArray();
            int len = cps.length;
            if (len < idStart) continue;

            String name = codePointSlice(cps, 0, idStart).trim();
            String id = codePointSlice(cps, idStart, Math.min(verStart, len)).trim();
            String version = len > verStart ? codePointSlice(cps, verStart, Math.min(srcStart, len)).trim() : "";

            if (!name.isEmpty() && !id.isEmpty()) {
                results.add(new SearchResult(name, id, version));
            }
        }
        return results;
    }

    private static List<SearchResult> parseScoopSearch(String output) {
        List<SearchResult> results = new ArrayList<>();
        int[] headerOffsets = null; // {verStart, srcStart}

        for (String raw : lines(output)) {
            String line = TextUtil.sanitizeLine(raw);
            if (line.trim().isEmpty() || line.startsWith("Results from") || isDashSeparatorLine(line)) continue;

            if (headerOffsets == null) {
                String lower = line.toLowerCase(Locale.ROOT);
                if (lower.contains("name") && lower.contains("version")) {
                    int verPos = orElseFindCol(line, "Version", "version");
                    int srcPos = orElseFindCol(line, "Source", "source");
                    if (verPos >= 0) {
                        headerOffsets = new int[] { verPos, srcPos >= 0 ? srcPos : Integer.MAX_VALUE };
                    }
                }
                continue;
            }

            int verStart = headerOffsets[0];
            int srcStart = headerOffsets[1];
            int len = line.length();
            if (len < verStart) continue;
            String name = line.substring(0, verStart).trim();
            String version = line.substring(verStart, Math.min(srcStart, len)).trim();
            if (!name.isEmpty()) {
                // scoop id == name (no separate winget-style ID)
                results.add(new SearchResult(name, name, version));
            }
        }
        return results;
    }

    private static List<SearchResult> parseChocoSearch(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = raw.trim();
            if (line.isEmpty()) continue;
            int sep = line.indexOf('|');
            String name = (sep >= 0 ? line.substring(0, sep) : line).trim();
            String version = sep >= 0 ? line.substring(sep + 1).trim() : "";
            if (name.isEmpty()) continue;
            results.add(new SearchResult(name, name, version));
        }
        return results;
    }

    private static List<SearchResult> parsePacmanSearch(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = TextUtil.sanitizeLine(raw);
            if (line.startsWith("::") || line.trim().isEmpty()
                    || line.startsWith(" ") || line.startsWith("\t")) continue;
            int sep = line.indexOf('/');
            String rest = (sep >= 0 ? line.substring(sep + 1) : line).trim();
            String[] tokens = whitespaceTokens(rest);
            String name = tokens.length > 0 ? tokens[0] : "";
            String version = tokens.length > 1 ? tokens[1] : "";
            if (!name.isEmpty()) {
                results.add(new SearchResult(name, name, version));
            }
        }
        return results;
    }

    private static List<SearchResult> parseAptSearch(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = TextUtil.sanitizeLine(raw);
            if (line.trim().isEmpty() || line.startsWith(" ") || line.startsWith("\t")
                    || line.startsWith("Sorting") || line.startsWith("Full Text")
                    || line.startsWith("WARNING")) continue;
            int sep = line.indexOf('/');
            String name = (sep >= 0 ? line.substring(0, sep) : line).trim();
            String afterSlash = sep >= 0 ? line.substring(sep + 1) : "";
            String[] tokens = whitespaceTokens(afterSlash.trim());
            String version = tokens.length > 0 ? tokens[0] : "";
            if (!name.isEmpty()) {
                results.add(new SearchResult(name, name, version));
            }
        }
        return results;
    }

    private static List<SearchResult> parseDnfSearch(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = TextUtil.sanitizeLine(raw);
            if (line.trim().isEmpty() || line.startsWith("Last metadata")
                    || line.startsWith("=====") || line.startsWith("Error")) continue;
            if (!line.contains(".x86_64") && !line.contains(".noarch") && !line.contains(".src")) continue;
            String[] tokens = whitespaceTokens(line.trim());
            String fullName = tokens.length > 0 ? tokens[0] : "";
            String version = tokens.length > 1 ? tokens[1] : "";
            String name = fullName.contains(".") ? fullName.substring(0, fullName.indexOf('.')) : fullName;
            if (!name.isEmpty()) {
                results.add(new SearchResult(name, name, version));
            }
        }
        return results;
    }

    private static List<SearchResult> parseXbpsSearch(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = raw.trim();
            if (line.isEmpty()) continue;
            String rest = (line.startsWith("[-]") || line.startsWith("[*]")) ? line.substring(3).trim() : line;
            String[] tokens = whitespaceTokens(rest);
            String pkg = tokens.length > 0 ? tokens[0] : "";
            String[] nv = TextUtil.splitPkgNameVersion(pkg);
            if (nv[0].isEmpty()) continue;
            results.add(new SearchResult(nv[0], nv[0], nv[1]));
        }
        return results;
    }

    private static List<SearchResult> parseApkSearch(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = raw.trim();
            if (line.isEmpty()) continue;
            String[] tokens = whitespaceTokens(line);
            String pkg = tokens.length > 0 ? tokens[0] : "";
            String[] nv = TextUtil.splitPkgNameVersion(pkg);
            if (nv[0].isEmpty()) continue;
            results.add(new SearchResult(nv[0], nv[0], nv[1]));
        }
        return results;
    }

    // ─── Installed-list parsers ────────────────────────────────────────────

    /** {@code pacman -Q} / {@code yay -Q} / {@code paru -Q} — "name version" per line. */
    private static List<SearchResult> parsePacmanList(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = raw.trim();
            if (line.isEmpty()) continue;
            String[] tokens = whitespaceTokens(line);
            if (tokens.length == 0 || tokens[0].isEmpty()) continue;
            String name = tokens[0];
            String version = tokens.length > 1 ? tokens[1] : "";
            results.add(new SearchResult(name, name, version));
        }
        return results;
    }

    /** {@code apt list --installed} — "pkg/suite now version arch [installed]". */
    private static List<SearchResult> parseAptList(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("Listing")) continue;
            int sep = line.indexOf('/');
            String name = (sep >= 0 ? line.substring(0, sep) : line).trim();
            if (name.isEmpty()) continue;
            String[] tokens = whitespaceTokens(line);
            String version = tokens.length > 1 ? tokens[1] : "";
            results.add(new SearchResult(name, name, version));
        }
        return results;
    }

    /** {@code dnf list installed} — "name.arch version @repo". */
    private static List<SearchResult> parseDnfList(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("Installed")
                    || line.startsWith("Last metadata") || line.startsWith("Available")) continue;
            String[] tokens = whitespaceTokens(line);
            if (tokens.length == 0) continue;
            String fullName = tokens[0];
            String version = tokens.length > 1 ? tokens[1] : "";
            String name = fullName.contains(".") ? fullName.substring(0, fullName.indexOf('.')) : fullName;
            if (name.isEmpty()) continue;
            results.add(new SearchResult(name, name, version));
        }
        return results;
    }

    /** {@code xbps-query -l} — "[*] pkg-ver description". */
    private static List<SearchResult> parseXbpsList(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = raw.trim();
            if (line.isEmpty()) continue;
            String rest = (line.startsWith("[*]") || line.startsWith("[-]")) ? line.substring(3).trim() : line;
            String[] tokens = whitespaceTokens(rest);
            String pkg = tokens.length > 0 ? tokens[0] : "";
            String[] nv = TextUtil.splitPkgNameVersion(pkg);
            if (nv[0].isEmpty()) continue;
            results.add(new SearchResult(nv[0], nv[0], nv[1]));
        }
        return results;
    }

    /** {@code apk list --installed} — same format as apk search. */
    private static List<SearchResult> parseApkList(String output) {
        return parseApkSearch(output);
    }

    /** {@code brew list --formula} — one package name per line. */
    private static List<SearchResult> parseBrewList(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String name = raw.trim();
            if (name.isEmpty()) continue;
            results.add(new SearchResult(name, name, ""));
        }
        return results;
    }

    /** {@code choco list --local-only} — "name version" per data line. */
    private static List<SearchResult> parseChocoList(String output) {
        List<SearchResult> results = new ArrayList<>();
        for (String raw : lines(output)) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("Chocolatey") || line.contains("packages installed")) continue;
            String[] tokens = whitespaceTokens(line);
            if (tokens.length == 0 || tokens[0].isEmpty()) continue;
            String name = tokens[0];
            String version = tokens.length > 1 ? tokens[1] : "";
            results.add(new SearchResult(name, name, version));
        }
        return results;
    }

    // ─── Helpers ───────────────────────────────────────────────────────────

    /** Mirrors Rust's {@code str::lines()}: splits on "\n"/"\r\n", never on a bare "\r". */
    private static String[] lines(String text) {
        return text.split("\r\n|\n");
    }

    /** Mirrors Rust's {@code split_whitespace()}: no empty tokens, ignores leading/trailing runs. */
    private static String[] whitespaceTokens(String s) {
        String trimmed = s.trim();
        return trimmed.isEmpty() ? new String[0] : trimmed.split("\\s+");
    }

    private static int orElseFindCol(String haystack, String preferred, String fallback) {
        int pos = TextUtil.findCol(haystack, preferred);
        return pos >= 0 ? pos : TextUtil.findCol(haystack, fallback);
    }

    private static boolean isDashSeparatorLine(String line) {
        for (int i = 0; i < line.length(); i++) {
            char c = line.charAt(i);
            if (c != '-' && c != ' ') return false;
        }
        return true;
    }

    private static String codePointSlice(int[] codePoints, int from, int to) {
        return new String(codePoints, from, to - from);
    }
}
