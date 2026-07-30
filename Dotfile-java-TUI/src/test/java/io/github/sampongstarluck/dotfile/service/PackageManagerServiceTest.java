package io.github.sampongstarluck.dotfile.service;

import io.github.sampongstarluck.dotfile.model.LinuxDistro;
import io.github.sampongstarluck.dotfile.model.OperatingSystem;
import io.github.sampongstarluck.dotfile.model.PackageManager;
import io.github.sampongstarluck.dotfile.service.implementation.PackageManagerServiceImp;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;

class PackageManagerServiceTest {

    private PackageManagerServiceImp service;

    @BeforeEach
    void setUp() {
        service = new PackageManagerServiceImp(mock(OsService.class), mock(PathService.class));
    }

    @Test
    void windowsCandidatesAreWingetScoopChoco() {
        var os = new OperatingSystem(OperatingSystem.Kind.WINDOWS, null, null);
        assertThat(service.candidatesFor(os))
                .containsExactly(PackageManager.WINGET, PackageManager.SCOOP, PackageManager.CHOCO);
    }

    @Test
    void archCandidatesArePacmanYayParu() {
        var os = new OperatingSystem(OperatingSystem.Kind.LINUX, LinuxDistro.ARCH, null);
        assertThat(service.candidatesFor(os))
                .containsExactly(PackageManager.PACMAN, PackageManager.YAY, PackageManager.PARU);
    }

    @Test
    void unknownDistroLinuxProbesFiveCandidates() {
        var os = new OperatingSystem(OperatingSystem.Kind.LINUX, null, null);
        assertThat(service.candidatesFor(os)).hasSize(5)
                .containsExactly(PackageManager.APT, PackageManager.DNF, PackageManager.PACMAN,
                        PackageManager.YAY, PackageManager.XBPS);
    }
}
