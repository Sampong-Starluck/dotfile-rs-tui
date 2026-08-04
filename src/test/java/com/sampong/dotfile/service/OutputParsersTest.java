package com.sampong.dotfile.service;

import com.sampong.dotfile.model.SearchResult;
import org.junit.jupiter.api.Test;

import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;

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

    /** Literal captured {@code winget upgrade} output (PLAN.md phase-13 §13.1), not a synthetic
     *  fixture — pasted verbatim from a real run with 16 pending updates. */
    @Test
    void wingetUpgradeParsesRealCapturedOutputIntoIdToAvailableVersionMap() {
        String text = "Name                             Id                                  Version                       Available                     Source\n"
                + "---------------------------------------------------------------------------------------------------------------------------------------\n"
                + "DBeaver 26.1.2 (current user)    DBeaver.DBeaver.Community           26.1.2                        26.1.3                        winget\n"
                + "Deno                             DenoLand.Deno                       2.9.0                         2.9.4                         winget\n"
                + "Docker Desktop                   Docker.DockerDesktop                4.82.0                        4.84.0                        winget\n"
                + "FFmpeg for yt-dlp                yt-dlp.FFmpeg                       N-123778-g3b55818764-20260331 N-125365-g9a01c1cb6a-20260630 winget\n"
                + "Kubernetes CLI                   Kubernetes.kubectl                  1.36.0                        1.36.3                        winget\n"
                + "lazygit                          JesseDuffield.lazygit               0.63.0                        0.63.1                        winget\n"
                + "OBS Studio                       OBSProject.OBSStudio                32.1.2                        32.2.1                        winget\n"
                + "Obsidian                         Obsidian.Obsidian                   1.12.7                        1.13.4                        winget\n"
                + "Oh My Posh                       JanDeDobbeleer.OhMyPosh             29.27.0.0                     30.0.0                        winget\n"
                + "PostgreSQL 18                    PostgreSQL.PostgreSQL.18            18.4-1                        18.4-2                        winget\n"
                + "PremiumSoft Navicat Premium 17.3 PremiumSoft.NavicatPremium          17.3.10                       17.3.11                       winget\n"
                + "Python 3.14.4 (64-bit)           Python.Python.3.14                  3.14.4                        3.14.6                        winget\n"
                + "Visual Studio Professional 2026  Microsoft.VisualStudio.Professional 18.8.0                        18.8.2                        winget\n"
                + "Windows Subsystem for Linux      Microsoft.WSL                       2.7.10.0                      2.7.11                        winget\n"
                + "Zed                              ZedIndustries.Zed                   1.10.2                        1.13.1                        winget\n"
                + "Zoom Workplace (64-bit)          Zoom.Zoom                           7.1.41345                    7.1.43453                     winget\n"
                + "16 upgrades available.\n";

        Map<String, String> updates = OutputParsers.parseUpgradeOutput("winget", text);

        assertThat(updates).hasSize(16);
        assertThat(updates).containsEntry("DBeaver.DBeaver.Community", "26.1.3");
        assertThat(updates).containsEntry("JesseDuffield.lazygit", "0.63.1");
        assertThat(updates).containsEntry("yt-dlp.FFmpeg", "N-125365-g9a01c1cb6a-20260630");
        assertThat(updates).containsEntry("Microsoft.WSL", "2.7.11");
        assertThat(updates).doesNotContainKey("16");
    }

    @Test
    void upgradeOutputDegradesToEmptyMapForUnsupportedManagers() {
        assertThat(OutputParsers.parseUpgradeOutput("apt", "anything")).isEmpty();
        assertThat(OutputParsers.parseUpgradeOutput("brew", "anything")).isEmpty();
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
