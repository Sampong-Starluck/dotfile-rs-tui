package com.sampong.dotfile.ui.state;

import com.sampong.dotfile.model.ShellStatus;
import org.jspecify.annotations.Nullable;

import java.util.ArrayList;
import java.util.List;

/** Shells panel + SHELL_INFO main view. {@code shells} null means "reload on next render" (Phase 8). */
public class ScriptsState {
    public int shellCursor = 0;
    public final List<String> log = new ArrayList<>();
    public @Nullable List<ShellStatus> shells = null;
    /** Effective primary shell: explicit config value, else the detected system default. */
    public @Nullable String primaryShell = null;
    /** The *explicit* {@code config.json} value only (null when falling back to system default) —
     *  lets the view distinguish "★ set as primary" from "◇ system default" without calling
     *  {@code ScriptService} itself (PLAN.md §4 SRP — views are zero-service-call). */
    public @Nullable String explicitPrimaryShell = null;
}
