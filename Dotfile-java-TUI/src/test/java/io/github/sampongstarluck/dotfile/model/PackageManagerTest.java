package io.github.sampongstarluck.dotfile.model;

import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class PackageManagerTest {

    @Test
    void everyManagerHasNonEmptyCommands() {
        for (PackageManager pm : PackageManager.values()) {
            assertThat(pm.commands()).as(pm.name()).isNotEmpty();
        }
    }

    @Test
    void xbpsBinaryIsXbpsInstall() {
        assertThat(PackageManager.XBPS.binary()).isEqualTo("xbps-install");
    }
}
