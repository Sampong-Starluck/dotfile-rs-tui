package com.sampong.dotfile.state;

import com.sampong.dotfile.model.ShellStatus;

import java.util.ArrayList;
import java.util.List;

/** Shells panel + SHELL_INFO main view. {@code shells} null means "reload on next render" (Phase 8). */
public class ScriptsState {
    public int shellCursor = 0;
    public final List<String> log = new ArrayList<>();
    public List<ShellStatus> shells = null;
    public String primaryShell = null;
}
