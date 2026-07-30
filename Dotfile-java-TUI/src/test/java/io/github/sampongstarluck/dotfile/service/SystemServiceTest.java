package io.github.sampongstarluck.dotfile.service;

import io.github.sampongstarluck.dotfile.service.implementation.SystemServiceImp;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class SystemServiceTest {

    private SystemService service;

    @BeforeEach
    void setUp() {
        service = new SystemServiceImp();
    }

    @Test
    void requiresSudoMatchesTheRustList() {
        assertThat(service.requiresSudo("pacman")).isTrue();
        assertThat(service.requiresSudo("winget")).isFalse();
    }

    @Test
    void requiresInteractiveMatchesTheRustList() {
        assertThat(service.requiresInteractive("choco")).isTrue();
        assertThat(service.requiresInteractive("brew")).isFalse();
        assertThat(service.requiresInteractive("scoop")).isFalse();
    }
}
