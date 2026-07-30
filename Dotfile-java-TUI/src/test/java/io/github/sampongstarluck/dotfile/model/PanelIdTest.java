package io.github.sampongstarluck.dotfile.model;

import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class PanelIdTest {

    @Test
    void nextCyclesThroughAllValuesAndWrapsAround() {
        PanelId p = PanelId.values()[0];
        for (int i = 1; i < PanelId.values().length; i++) {
            p = p.next();
            assertThat(p).isEqualTo(PanelId.values()[i]);
        }
        assertThat(p.next()).isEqualTo(PanelId.values()[0]);
    }

    @Test
    void prevCyclesThroughAllValuesAndWrapsAround() {
        PanelId p = PanelId.values()[0];
        for (int i = PanelId.values().length - 1; i >= 1; i--) {
            p = p.prev();
            assertThat(p).isEqualTo(PanelId.values()[i]);
        }
        assertThat(p.prev()).isEqualTo(PanelId.values()[0]);
    }

    @Test
    void nextAndPrevAreInverses() {
        for (PanelId p : PanelId.values()) {
            assertThat(p.next().prev()).isEqualTo(p);
            assertThat(p.prev().next()).isEqualTo(p);
        }
    }
}
