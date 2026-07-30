package com.sampong.dotfile.state;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.LinkedBlockingQueue;

/** Streaming install/remove log + interactive stdin + suspend-TUI trampoline (Phase 9). */
public class InstallState {
    public final List<String> log = new ArrayList<>();
    public ConcurrentLinkedQueue<String> logQueue = null;
    public LinkedBlockingQueue<String> stdinQueue = null;
    public final List<String> runExternal = new ArrayList<>();
    public boolean runExternalRemoving = false;
}
