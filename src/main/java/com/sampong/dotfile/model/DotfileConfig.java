package com.sampong.dotfile.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import org.jspecify.annotations.Nullable;

/**
 * Port of Rust {@code service/script_service.rs::DotfileConfig}. snake_case on disk =
 * byte-compatible with the Rust app's {@code config.json}. {@code primaryShell} is absent
 * from {@code config.json} until a primary shell has been set (Phase 8).
 */
@JsonIgnoreProperties(ignoreUnknown = true)
public record DotfileConfig(@JsonProperty("primary_shell") @Nullable String primaryShell) {}
