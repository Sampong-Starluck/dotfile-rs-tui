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
    public @Nullable String primaryShell = null;
}
