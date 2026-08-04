package com.sampong.dotfile.service;

import com.sampong.dotfile.model.LinuxDistro;
import com.sampong.dotfile.model.OperatingSystem;
import com.sampong.dotfile.model.PackageManager;
import com.sampong.dotfile.service.implementation.PackageManagerServiceImp;
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
