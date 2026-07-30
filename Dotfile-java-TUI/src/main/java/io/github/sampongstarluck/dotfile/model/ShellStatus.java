package io.github.sampongstarluck.dotfile.model;

import java.nio.file.Path;

/** Port of Rust {@code service/script_service.rs::ShellStatus}. Detection is Phase 8. */
public record ShellStatus(ShellEntry entry, boolean detected, boolean deployed, Path targetPath) {}
