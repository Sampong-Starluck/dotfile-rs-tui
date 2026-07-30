package com.sampong.dotfile.ui.state;

import org.jspecify.annotations.Nullable;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.LinkedBlockingQueue;

/** Streaming install/remove log + interactive stdin + suspend-TUI trampoline (Phase 9). */
public class InstallState {
    public final List<String> log = new ArrayList<>();
    public @Nullable ConcurrentLinkedQueue<String> logQueue = null;
    public @Nullable LinkedBlockingQueue<String> stdinQueue = null;
    public final List<String> runExternal = new ArrayList<>();
    public boolean runExternalRemoving = false;
}
