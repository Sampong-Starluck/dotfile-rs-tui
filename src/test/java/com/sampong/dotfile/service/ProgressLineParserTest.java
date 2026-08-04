package com.sampong.dotfile.service;

import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

/** Cases built from literal captured {@code winget update}/{@code winget install} output
 *  (PLAN.md phase-12 §12.1) — not synthetic fixtures. */
class ProgressLineParserTest {

    @Test
    void parsesDownloadSizeLine() {
        var progress = ProgressLineParser.parse("  ██████████████████████████████████   29.2 MB / 29.2 MB")
                .orElseThrow();
        assertThat(progress.downloadedText()).isEqualTo("29.2 MB");
        assertThat(progress.totalText()).isEqualTo("29.2 MB");
        assertThat(progress.percent()).isEqualTo(100);
    }

    @Test
    void computesPartialDownloadPercent() {
        var progress = ProgressLineParser.parse("  ███████████                  12.1 MB / 38.6 MB")
                .orElseThrow();
        assertThat(progress.downloadedText()).isEqualTo("12.1 MB");
        assertThat(progress.totalText()).isEqualTo("38.6 MB");
        assertThat(progress.percent()).isEqualTo(31);
    }

    @Test
    void parsesInstallPercentOnlyLine() {
        var progress = ProgressLineParser.parse("  ██████████████████████████████████   100%").orElseThrow();
        assertThat(progress.downloadedText()).isNull();
        assertThat(progress.totalText()).isNull();
        assertThat(progress.percent()).isEqualTo(100);
    }

    @Test
    void mixedUnitsConvertBeforeComputingPercent() {
        var progress = ProgressLineParser.parse("512 KB / 2 MB").orElseThrow();
        assertThat(progress.percent()).isEqualTo(25);
    }

    @Test
    void nonProgressLinesDoNotParse() {
        assertThat(ProgressLineParser.parse("Successfully installed")).isEmpty();
        assertThat(ProgressLineParser.parse("-")).isEmpty();
        assertThat(ProgressLineParser.parse("Downloading https://example.com/x.exe")).isEmpty();
    }
}
