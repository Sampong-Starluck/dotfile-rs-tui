package io.github.sampongstarluck.dotfile.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import java.util.List;

/**
 * Port of Rust {@code models/shell_model.rs::ShellEntry}. {@code shells.json} carries extra
 * fields ({@code function}, {@code version}, {@code lastUpdated}) — ignored here.
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public record ShellEntry(
        String id, String name, boolean hidden, String description,
        int order, List<String> platforms, List<String> requires) {
    public ShellEntry {
        if (description == null) description = "";
        if (platforms == null) platforms = List.of();
        if (requires == null) requires = List.of();
    }
}
