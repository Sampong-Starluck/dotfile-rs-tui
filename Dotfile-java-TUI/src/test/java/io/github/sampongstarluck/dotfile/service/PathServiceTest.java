package io.github.sampongstarluck.dotfile.service;

import io.github.sampongstarluck.dotfile.service.implementation.PathServiceImp;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledOnOs;
import org.junit.jupiter.api.condition.OS;

import static org.assertj.core.api.Assertions.assertThat;

class PathServiceTest {

    private PathService service;

    @BeforeEach
    void setUp() {
        service = new PathServiceImp();
    }

    @Test
    @EnabledOnOs(OS.WINDOWS)
    void whichFindsCmdOnWindows() {
        assertThat(service.which("cmd"))
                .isPresent()
                .get()
                .satisfies(p -> assertThat(p.toString().toLowerCase()).endsWith("cmd.exe"));
    }

    @Test
    void whichReturnsEmptyForUnknownBinary() {
        assertThat(service.which("definitely-not-a-binary-xyz")).isEmpty();
    }
}
