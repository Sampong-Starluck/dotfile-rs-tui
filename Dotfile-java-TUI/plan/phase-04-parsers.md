# Phase 4 — Output decoding + all package-manager parsers

**Goal:** byte-perfect ports of `../src/utils/decode_util.rs`,
`../src/utils/text_util.rs`, and every parser in
`../src/service/search_service.rs` (9 search parsers + 8 list parsers).
These are the highest-bug-risk part of the app — port them carefully and
test them with the captured samples below.

All static methods. Files: `service/DecodeUtil.java`, `service/TextUtil.java`,
`service/OutputParsers.java`.

## 4.1 `DecodeUtil` (port of `decode_util.rs`)

```java
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

    /** Port of is_noise_line(): filter winget spinner/progress junk. Exact rules: */
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
```

## 4.2 `TextUtil` (port of `text_util.rs`)

```java
public final class TextUtil {
    private TextUtil() {}

    /** Remove ANSI escape sequences (ESC[ ... letter, or ESC+1 char). */
    public static String stripAnsi(String s) {
        StringBuilder out = new StringBuilder(s.length());
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c == 0x1b) {
                if (i + 1 < s.length() && s.charAt(i + 1) == '[') {
                    i++;                                    // consume '['
                    while (i + 1 < s.length()) {
                        i++;
                        char sc = s.charAt(i);
                        if (Character.isLetter(sc) && sc < 128) break;
                    }
                } else {
                    i++;                                    // consume one char
                }
            } else {
                out.append(c);
            }
        }
        return out.toString();
    }

    public static String sanitizeLine(String line) {
        return line.endsWith("\r") ? line.substring(0, line.length() - 1) : line;
    }

    /** char-index (NOT byte) of needle in haystack — for column detection. */
    public static int findCol(String haystack, String needle) {
        return haystack.indexOf(needle);      // Java String.indexOf is char-based: correct
    }                                          // return -1 when absent (Rust returned None)

    /** "neovim-0.9.5_1" → ["neovim", "0.9.5_1"]; version starts at last '-' followed by digit. */
    public static String[] splitPkgNameVersion(String pkg) {
        for (int i = pkg.length() - 1; i >= 1; i--) {
            if (pkg.charAt(i - 1) == '-' && Character.isDigit(pkg.charAt(i))) {
                return new String[] { pkg.substring(0, i - 1), pkg.substring(i) };
            }
        }
        return new String[] { pkg, "" };
    }
}
```

## 4.3 `OutputParsers` — dispatchers

Port the two dispatchers from `search_service.rs:39` and `:69` exactly:

```java
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
    // ... individual parsers below
}
```

## 4.4 Individual parsers — port rules

Translate each `fn parse_*` from `../src/service/search_service.rs` 1:1.
Key mechanics to preserve **exactly**:

### `parseWingetSearch` (search_service.rs:113 — the trickiest)
1. Split the whole output on **both** `\r` and `\n` (`text.split("[\\r\\n]")`)
   — winget animates with bare `\r`. Apply `TextUtil.stripAnsi` to every segment.
2. Skip: blank lines; lines containing `█` or `▒` or `KB /` or `MB /`;
   spinner frames where trimmed line is `-`, `\`, `|`, `/`.
3. Header detection (before any data row): lowercase line containing
   `name` + `id` + `version` → record char columns of `Id`, `Version`,
   `Source` via `findCol` (try exact case then lowercase). Require
   `idCol > 0 && verCol > idCol`. `Source` missing → `Integer.MAX_VALUE`.
4. Skip separator lines (all chars `-` or space).
5. Data rows: slice by char index — `name = line[0..idCol]`,
   `id = line[idCol..min(verCol,len)]`, `version = line[verCol..min(srcCol,len)]`,
   all `.trim()`ed; skip when name or id empty. **Use code-point-safe slicing**:
   the Rust code collects `chars()`; in Java use
   `line.codePoints().toArray()` and build substrings from that array, because
   winget rows can contain non-BMP characters and column offsets are in chars.

### `parseScoopSearch` (…:181)
Header = line with `name` + `version` → columns of `Version` and `Source`.
Skip `Results from...` lines and dash separators. Rows: name = `[0..verCol]`,
version = `[verCol..srcCol]`; `id = name`.

### `parseChocoSearch` (…:220)
Each line `name|version` (because we pass `--limit-output`). Split on first `|`.

### `parsePacmanSearch` (…:236)
Skip lines starting with `::`, blank, or indented (description lines).
`repo/name version …` → split on first `/`, then whitespace-split the rest:
token0 = name, token1 = version. `id = name`.

### `parseAptSearch` (…:256)
Skip blank, indented, `Sorting`, `Full Text`, `WARNING` lines.
`name/suite version …` → name before first `/`; version = first
whitespace-token after the `/`.

### `parseDnfSearch` (…:274)
Only lines containing `.x86_64` | `.noarch` | `.src`. token0 = full name
(strip from first `.`), token1 = version.

### `parseXbpsSearch` / `parseApkSearch` (…:293, :311)
Strip optional `[-]` / `[*]` prefix; first whitespace token →
`splitPkgNameVersion`.

### List parsers (…:329–435)
- `parsePacmanList` — `name version` per line.
- `parseAptList` — skip `Listing`; name before `/`; version = 2nd token.
- `parseDnfList` — skip `Installed`/`Last metadata`/`Available` headers;
  `name.arch version` → strip arch suffix.
- `parseXbpsList` — like xbps search.
- `parseApkList` — alias of apk search parser.
- `parseBrewList` — one bare name per line, empty version.
- `parseChocoList` — skip lines starting `Chocolatey` or containing
  `packages installed`; `name version` tokens.

Every parser must call `sanitizeLine`/trim `\r` exactly where the Rust does.

## 4.5 Unit tests — REQUIRED fixtures

Create `src/test/resources/fixtures/` and tests that assert exact
`SearchResult` values. Embed these fixture strings in the test class (they are
distilled from real output):

**winget-search.txt** (note: build the test bytes as UTF-16LE with BOM for the
decode test, and plain string for the parse test)
```
   -
   \
Name             Id                      Version   Source
---------------------------------------------------------------
PowerShell       Microsoft.PowerShell    7.4.1     winget
Git              Git.Git                 2.45.0    winget
```
→ decode(UTF-16LE bytes) == text; parse → 2 rows;
row0 = ("PowerShell", "Microsoft.PowerShell", "7.4.1").

**choco-search.txt**
```
git|2.45.0
7zip|23.1.0
```
→ 2 rows, id==name.

**pacman-search.txt**
```
extra/git 2.45.0-1
    Fast distributed version control system
core/zsh 5.9-4
    A very advanced and programmable command interpreter
```
→ 2 rows ("git","2.45.0-1"), ("zsh","5.9-4"); description lines skipped.

**apt-search.txt**
```
Sorting...
Full Text Search...
git/noble 1:2.43.0-1ubuntu7 amd64
  fast, scalable, distributed revision control system
```
→ 1 row ("git", "1:2.43.0-1ubuntu7").

**dnf-search.txt**
```
Last metadata expiration check: 0:12:34 ago.
==================== Name Exactly Matched: git ====================
git.x86_64 : Fast Version Control System
```
→ 1 row, name "git" (note: version here ends up being `":"` — match the Rust
behavior exactly, don't "fix" it).

**xbps-search.txt**
```
[-] git-2.45.0_1        Git version control
[*] zsh-5.9_4           Z shell
```
→ ("git","2.45.0_1") and ("zsh","5.9_4").

**pacman-list.txt** → `git 2.45.0-1` lines. **brew-list.txt** → bare names.
**choco-list.txt**
```
Chocolatey v2.3.0
git 2.45.0
7zip 23.1.0
3 packages installed.
```
→ 2 rows.

Also test `TextUtil.stripAnsi("[32mOK[0m done") == "OK done"`,
`splitPkgNameVersion("neovim-0.9.5_1") == ["neovim","0.9.5_1"]`,
`splitPkgNameVersion("git") == ["git",""]`, and 5 `isNoiseLine` cases
(`"", "-", "  45%", "██ 3 MB / 10 MB", "real line"→false`).

## Definition of Done (Phase 4)

- [ ] All parsers implemented; `OutputParsers` has no I/O, no Spring, no TamboUI — pure static functions
- [ ] Every fixture test above written and green (`mise run test`)
- [ ] Live smoke test (temporary main or test, then delete): run
      `winget search git`, capture stdout bytes with `Process.getInputStream().readAllBytes()`,
      run through `decodeSearchOutput` + `parseSearchOutput("winget", …)` and
      log the row count — must be > 0 on this machine
