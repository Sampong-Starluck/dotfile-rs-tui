package com.sampong.dotfile.ui.state;

import com.sampong.dotfile.model.InstallKind;
import org.jspecify.annotations.Nullable;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.LinkedBlockingQueue;

/** Streaming install/remove log + interactive stdin + suspend-TUI trampoline (Phase 9). */
public class InstallState {
    public @Nullable InstallKind kind = null;
    public final List<String> log = new ArrayList<>();
    public @Nullable ConcurrentLinkedQueue<String> logQueue = null;
    public @Nullable LinkedBlockingQueue<String> stdinQueue = null;
    public final List<String> runExternal = new ArrayList<>();
    public boolean runExternalRemoving = false;

    /** Display names of the app(s) targeted by the current run, set once by
     *  {@code InstallController.begin()}; null hides the detail line (Phase 12 §12.1). */
    public @Nullable String targetLabel = null;
    /** Parsed live download/install progress, drained from {@code InstallLogBridge} each frame.
     *  {@code progressActive} (not {@code progressPercent > 0}, since 0% is itself a valid
     *  in-progress reading) is what hides/shows the bar (Phase 12 §12.1). */
    public boolean progressActive = false;
    public @Nullable String progressLabel = null;
    public @Nullable String progressDownloaded = null;
    public @Nullable String progressTotal = null;
    public int progressPercent = 0;
}
