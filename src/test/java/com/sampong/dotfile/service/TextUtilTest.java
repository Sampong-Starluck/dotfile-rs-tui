package com.sampong.dotfile.service;

import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class TextUtilTest {

    @Test
    void stripAnsiRemovesEscapeSequences() {
        String input = "[32mOK[0m done";
        assertThat(TextUtil.stripAnsi(input)).isEqualTo("OK done");
    }

    @Test
    void sanitizeLineStripsTrailingCr() {
        assertThat(TextUtil.sanitizeLine("hello\r")).isEqualTo("hello");
        assertThat(TextUtil.sanitizeLine("hello")).isEqualTo("hello");
    }

    @Test
    void findColReturnsCharIndexOrMinusOne() {
        assertThat(TextUtil.findCol("Name  Id  Version", "Id")).isEqualTo(6);
        assertThat(TextUtil.findCol("Name  Id  Version", "Missing")).isEqualTo(-1);
    }

    @Test
    void splitPkgNameVersionSplitsAtLastHyphenBeforeDigit() {
        assertThat(TextUtil.splitPkgNameVersion("neovim-0.9.5_1")).containsExactly("neovim", "0.9.5_1");
        assertThat(TextUtil.splitPkgNameVersion("git")).containsExactly("git", "");
    }
}
