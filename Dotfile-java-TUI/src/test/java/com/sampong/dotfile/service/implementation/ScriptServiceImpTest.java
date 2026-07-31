package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.config.AppProperties;
import com.sampong.dotfile.model.DotfileConfig;
import com.sampong.dotfile.service.AppCatalogService;
import com.sampong.dotfile.service.OsService;
import com.sampong.dotfile.service.PathService;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import tools.jackson.databind.ObjectMapper;

import java.nio.file.Files;
import java.nio.file.Path;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/** Every test forces the Unix/XDG path branch (mocked {@code isWindows() == false}) and overrides
 *  {@code homeDir()} to {@code @TempDir} — the Windows branch resolves against the real
 *  {@code %APPDATA%} env var, which a unit test must never touch (PLAN.md phase-08 §8.2). */
class ScriptServiceImpTest {

    @TempDir
    Path home;

    private ScriptServiceImp service;

    @BeforeEach
    void setUp() {
        OsService osService = mock(OsService.class);
        when(osService.isWindows()).thenReturn(false);
        when(osService.isMac()).thenReturn(false);
        PathService pathService = mock(PathService.class);
        AppCatalogService appCatalogService = mock(AppCatalogService.class);

        service = new ScriptServiceImp(pathService, osService, appCatalogService,
                new ObjectMapper(), new AppProperties("dotfile-rs-test"));
        service.overrideHomeDirForTest(home);
    }

    @Test
    void deployWritesTheResourceContentThenUndeployRemovesIt() throws Exception {
        Path target = service.deployScript("bash");

        assertThat(target).exists();
        assertThat(Files.readString(target)).isEqualTo(service.scriptContent("bash"));

        service.undeployScript("bash");

        assertThat(target).doesNotExist();
    }

    @Test
    void addSourceToProfileCreatesTheBlockAndIsIdempotent() throws Exception {
        var first = service.addSourceToProfile("bash");

        assertThat(first.changed()).isTrue();
        String content = Files.readString(first.profile());
        assertThat(content).contains("# dotfile-rs");
        assertThat(content).contains(service.sourceHint("bash"));

        var second = service.addSourceToProfile("bash");

        assertThat(second.changed()).isFalse();
        assertThat(Files.readString(first.profile())).isEqualTo(content);
    }

    @Test
    void removeSourceFromProfileStripsOnlyTheBlockAndPreservesTrailingNewline() throws Exception {
        Path profile = home.resolve(".bashrc");
        String sourceLine = service.sourceHint("bash");
        String original = "export PATH=$PATH:/usr/local/bin\n"
                + "\n# dotfile-rs\n" + sourceLine + "\n"
                + "alias ll='ls -la'\n";
        Files.writeString(profile, original);

        var result = service.removeSourceFromProfile("bash");

        assertThat(result.changed()).isTrue();
        String cleaned = Files.readString(profile);
        assertThat(cleaned).isEqualTo(
                "export PATH=$PATH:/usr/local/bin\nalias ll='ls -la'\n");

        var second = service.removeSourceFromProfile("bash");
        assertThat(second.changed()).isFalse();
    }

    @Test
    void readWriteConfigRoundTrips() throws Exception {
        service.setPrimaryShell("zsh");

        DotfileConfig config = service.readConfig();

        assertThat(config.primaryShell()).isEqualTo("zsh");
    }

    @Test
    void corruptConfigFallsBackToDefaultWithoutThrowing() throws Exception {
        Path configPath = service.configPath();
        Files.createDirectories(configPath.getParent());
        Files.writeString(configPath, "{ not valid json");

        DotfileConfig config = service.readConfig();

        assertThat(config.primaryShell()).isNull();
    }

    @Test
    void sourceHintForPowershellUsesDotSourceSyntax() {
        assertThat(service.sourceHint("powershell")).startsWith(". \"");
    }
}
