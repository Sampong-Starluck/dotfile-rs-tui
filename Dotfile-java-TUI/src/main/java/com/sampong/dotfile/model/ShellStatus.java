package com.sampong.dotfile.model;

import org.jspecify.annotations.Nullable;

import java.nio.file.Path;

/**
 * Port of Rust {@code service/script_service.rs::ShellStatus}, widened with {@code binary}/
 * {@code profilePath}/{@code sourceHint} — pure, id-derived values the Rust UI computes inline
 * at render time via free functions. The Java {@code ui/} package may not call services
 * (PLAN.md §4 SRP — views are zero-service-call), so {@code ScriptServiceImp#loadShellStatuses}
 * computes them once here instead of {@code ShellInfoView} calling {@code ScriptService} itself.
 */
public record ShellStatus(ShellEntry entry, boolean detected, boolean deployed, @Nullable Path targetPath,
                           @Nullable String binary, @Nullable Path profilePath, @Nullable String sourceHint) {}
