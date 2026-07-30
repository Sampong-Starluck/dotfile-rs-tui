package io.github.sampongstarluck.dotfile.model;

import org.junit.jupiter.api.Test;
import tools.jackson.databind.ObjectMapper;

import static org.assertj.core.api.Assertions.assertThat;

class DotfileConfigTest {

    private final ObjectMapper mapper = new ObjectMapper();

    @Test
    void serializesUsingSnakeCaseKey() {
        String json = mapper.writeValueAsString(new DotfileConfig("zsh"));
        assertThat(json).isEqualTo("{\"primary_shell\":\"zsh\"}");
    }

    @Test
    void deserializesFromSnakeCaseKey() {
        DotfileConfig cfg = mapper.readValue("{\"primary_shell\":\"zsh\"}", DotfileConfig.class);
        assertThat(cfg.primaryShell()).isEqualTo("zsh");
    }
}
