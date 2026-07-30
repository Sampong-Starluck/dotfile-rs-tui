package io.github.sampongstarluck.dotfile.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Port of Rust {@code service/script_service.rs::DotfileConfig}. snake_case on disk =
 * byte-compatible with the Rust app's {@code config.json}.
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public record DotfileConfig(@JsonProperty("primary_shell") String primaryShell) {}
