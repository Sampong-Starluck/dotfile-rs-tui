package io.github.sampongstarluck.dotfile.service;

import io.github.sampongstarluck.dotfile.model.SearchResult;
import org.junit.jupiter.api.Test;

import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;
import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Fixture tests for {@link OutputParsers}, distilled from real package-manager output
 * per Phase 4 ({@code plan/phase-04-parsers.md}).
 */
class OutputParsersTest {

    @Test
    void wingetSearchDecodesUtf16AndParsesRows() throws Exception {
        String text = "   -\n   \\\nName             Id                      Version   Source\n"
                + "---------------------------------------------------------------\n"
                + "PowerShell       Microsoft.PowerShell    7.4.1     winget\n"
                + "Git              Git.Git                 2.45.0    winget\n";

        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        bytes.write(0xFF);
        bytes.write(0xFE);
        bytes.write(text.getBytes(StandardCharsets.UTF_16LE));

        String decoded = OutputParsers.decodeSearchOutput("winget", bytes.toByteArray());
        assertThat(decoded).isEqualTo(text);

        List<SearchResult> rows = OutputParsers.parseSearchOutput("winget", decoded);
        assertThat(rows).hasSize(2);
        assertThat(rows.get(0)).isEqualTo(new SearchResult("PowerShell", "Microsoft.PowerShell", "7.4.1"));
        assertThat(rows.get(1)).isEqualTo(new SearchResult("Git", "Git.Git", "2.45.0"));
    }

    @Test
    void chocoSearchSplitsOnPipeWithIdEqualToName() {
        String text = "git|2.45.0\n7zip|23.1.0\n";
        List<SearchResult> rows = OutputParsers.parseSearchOutput("choco", text);
        assertThat(rows).containsExactly(
                new SearchResult("git", "git", "2.45.0"),
                new SearchResult("7zip", "7zip", "23.1.0"));
    }

    @Test
    void pacmanSearchSkipsDescriptionLines() {
        String text = "extra/git 2.45.0-1\n"
                + "    Fast distributed version control system\n"
                + "core/zsh 5.9-4\n"
                + "    A very advanced and programmable command interpreter\n";
        List<SearchResult> rows = OutputParsers.parseSearchOutput("pacman", text);
        assertThat(rows).containsExactly(
                new SearchResult("git", "git", "2.45.0-1"),
                new SearchResult("zsh", "zsh", "5.9-4"));
    }

    /**
     * NOTE: {@code ../src/service/search_service.rs:parse_apt_search} splits the line on the
     * first '/' and takes the FIRST whitespace token of the remainder as "version" — for real
     * apt output ("pkg/suite version arch") that first token is the suite name, not the version.
     * This is a byte-perfect port of that (pre-existing) Rust behavior, not a bug fix — the
     * phase-04 plan doc's fixture answer ("git","1:2.43.0-1ubuntu7") does not match the actual
     * Rust source; verified by simulating splitn(2,'/') semantics. See STATUS in CLAUDE.md.
     */
    @Test
    void aptSearchMatchesActualRustBehaviorNotPlanFixture() {
        String text = "Sorting...\nFull Text Search...\n"
                + "git/noble 1:2.43.0-1ubuntu7 amd64\n"
                + "  fast, scalable, distributed revision control system\n";
        List<SearchResult> rows = OutputParsers.parseSearchOutput("apt", text);
        assertThat(rows).containsExactly(new SearchResult("git", "git", "noble"));
    }

    @Test
    void dnfSearchMatchesRustQuirkyVersionField() {
        String text = "Last metadata expiration check: 0:12:34 ago.\n"
                + "==================== Name Exactly Matched: git ====================\n"
                + "git.x86_64 : Fast Version Control System\n";
        List<SearchResult> rows = OutputParsers.parseSearchOutput("dnf", text);
        assertThat(rows).containsExactly(new SearchResult("git", "git", ":"));
    }

    @Test
    void xbpsSearchStripsMarkerPrefixAndSplitsNameVersion() {
        String text = "[-] git-2.45.0_1        Git version control\n"
                + "[*] zsh-5.9_4           Z shell\n";
        List<SearchResult> rows = OutputParsers.parseSearchOutput("xbps-install", text);
        assertThat(rows).containsExactly(
                new SearchResult("git", "git", "2.45.0_1"),
                new SearchResult("zsh", "zsh", "5.9_4"));
    }

    @Test
    void pacmanListParsesNameVersionPerLine() {
        String text = "git 2.45.0-1\nzsh 5.9-4\n";
        List<SearchResult> rows = OutputParsers.parseListOutput("pacman", text);
        assertThat(rows).containsExactly(
                new SearchResult("git", "git", "2.45.0-1"),
                new SearchResult("zsh", "zsh", "5.9-4"));
    }

    @Test
    void brewListParsesBareNamesWithEmptyVersion() {
        String text = "git\n7zip\n";
        List<SearchResult> rows = OutputParsers.parseListOutput("brew", text);
        assertThat(rows).containsExactly(
                new SearchResult("git", "git", ""),
                new SearchResult("7zip", "7zip", ""));
    }

    @Test
    void chocoListSkipsHeaderAndFooterLines() {
        String text = "Chocolatey v2.3.0\n"
                + "git 2.45.0\n"
                + "7zip 23.1.0\n"
                + "3 packages installed.\n";
        List<SearchResult> rows = OutputParsers.parseListOutput("choco", text);
        assertThat(rows).containsExactly(
                new SearchResult("git", "git", "2.45.0"),
                new SearchResult("7zip", "7zip", "23.1.0"));
    }
}
