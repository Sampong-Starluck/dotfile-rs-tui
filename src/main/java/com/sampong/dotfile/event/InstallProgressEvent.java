package com.sampong.dotfile.event;

import org.jspecify.annotations.Nullable;

/**
 * A parsed download/install progress frame, published by {@code InstallExecutionService}
 * (PLAN.md phase-12 §12.1) whenever a winget progress line carries a percentage — the "latest
 * wins" counterpart to {@link InstallLogEvent}'s queued lines. {@code cleared} is a dedicated
 * flag (not inferred from {@code label == null}, since a genuine progress line with no known
 * label yet — e.g. before any "Downloading …" line was seen — is legitimate data, not a
 * "hide the bar" signal) meaning "no active progress, hide the bar".
 */
public record InstallProgressEvent(
        @Nullable String label, @Nullable String downloadedText, @Nullable String totalText,
        int percent, boolean cleared) {

    public InstallProgressEvent(@Nullable String label, @Nullable String downloadedText,
                                 @Nullable String totalText, int percent) {
        this(label, downloadedText, totalText, percent, false);
    }

    public static InstallProgressEvent noProgress() {
        return new InstallProgressEvent(null, null, null, 0, true);
    }
}
