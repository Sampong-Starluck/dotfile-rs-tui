package io.github.sampongstarluck.dotfile.service;

import io.github.sampongstarluck.dotfile.model.AppEntry;
import io.github.sampongstarluck.dotfile.service.implementation.InstallCommandServiceImp;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Map;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class InstallCommandServiceTest {

    private OsService os;
    private InstallCommandService service;

    @BeforeEach
    void setUp() {
        os = mock(OsService.class);
        when(os.osKey()).thenReturn("windows");
        service = new InstallCommandServiceImp(os);
    }

    @Test
    void installCommandBuildsTheWingetLine() {
        assertThat(service.installCommand("winget", "Git.Git")).isEqualTo("winget install --id Git.Git -e");
    }

    @Test
    void removeCommandBuildsTheChocoLine() {
        assertThat(service.removeCommand("choco", "git")).isEqualTo("choco uninstall git -y");
    }

    @Test
    void installCommandForFallsBackToScoopWhenDetectedMgrIsMissing() {
        AppEntry entry = new AppEntry("Git", "git", Map.of("windows", Map.of("scoop", "git")));

        assertThat(service.installCommandFor(entry, "winget")).contains("scoop install git");
    }
}
