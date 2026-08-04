package com.sampong.dotfile.service;

import com.sampong.dotfile.model.AppEntry;
import com.sampong.dotfile.service.implementation.InstallCommandServiceImp;
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
    void updateCommandBuildsTheWingetLine() {
        assertThat(service.updateCommand("winget", "JesseDuffield.lazygit"))
                .isEqualTo("winget upgrade --id JesseDuffield.lazygit -e");
    }

    @Test
    void installCommandForFallsBackToScoopWhenDetectedMgrIsMissing() {
        AppEntry entry = new AppEntry("Git", "git", Map.of("windows", Map.of("scoop", "git")));

        assertThat(service.installCommandFor(entry, "winget")).contains("scoop install git");
    }
}
