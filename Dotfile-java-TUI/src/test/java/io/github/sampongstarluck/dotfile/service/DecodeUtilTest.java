package io.github.sampongstarluck.dotfile.service;

import org.junit.jupiter.api.Test;

import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;

import static org.assertj.core.api.Assertions.assertThat;

class DecodeUtilTest {

    @Test
    void decodesUtf16LeWithBom() throws Exception {
        String text = "Name             Id                      Version   Source\n"
                + "PowerShell       Microsoft.PowerShell    7.4.1     winget\n";
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        bytes.write(0xFF);
        bytes.write(0xFE);
        bytes.write(text.getBytes(StandardCharsets.UTF_16LE));

        assertThat(DecodeUtil.decodeWingetOutput(bytes.toByteArray())).isEqualTo(text);
    }

    @Test
    void decodesUtf8Bom() throws Exception {
        String text = "hello";
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        bytes.write(0xEF);
        bytes.write(0xBB);
        bytes.write(0xBF);
        bytes.write(text.getBytes(StandardCharsets.UTF_8));

        assertThat(DecodeUtil.decodeWingetOutput(bytes.toByteArray())).isEqualTo(text);
    }

    @Test
    void decodesPlainUtf8() {
        assertThat(DecodeUtil.decodeWingetOutput("plain text".getBytes(StandardCharsets.UTF_8)))
                .isEqualTo("plain text");
    }

    @Test
    void isNoiseLineCases() {
        assertThat(DecodeUtil.isNoiseLine("")).isTrue();
        assertThat(DecodeUtil.isNoiseLine("-")).isTrue();
        assertThat(DecodeUtil.isNoiseLine("  45%")).isTrue();
        assertThat(DecodeUtil.isNoiseLine("██ 3 MB / 10 MB")).isTrue();
        assertThat(DecodeUtil.isNoiseLine("real line")).isFalse();
    }
}
