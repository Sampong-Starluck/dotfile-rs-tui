package com.sampong.dotfile.ui;

import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.ui.state.AppState;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.util.List;

/**
 * Suspend-TUI execution for interactive package managers (port of {@code main.rs::run_in_terminal},
 * PLAN.md phase-09 §9.5). {@code TuiApp} invokes this between {@code ToolkitRunner} lifecycles,
 * after the terminal has already been fully restored by closing the runner — the only place
 * {@code System.out}/{@code System.in} are used in the app (CLAUDE.md hard rules).
 */
public final class ExternalRunner {

    private static final String CYAN = "[36m";
    private static final String GREEN = "[32m";
    private static final String RED = "[31m";
    private static final String YELLOW = "[33m";
    private static final String RESET = "[0m";

    private ExternalRunner() {
    }

    public static void run(AppState st) {
        List<String> commands = List.copyOf(st.install.runExternal);
        st.install.runExternal.clear();
        boolean wasRemoving = st.install.runExternalRemoving;
        st.install.runExternalRemoving = false;

        System.out.printf("%n%s  Running %d command(s)%s%n", CYAN, commands.size(), RESET);
        for (String cmd : commands) {
            System.out.println("\n" + CYAN + "▶  " + cmd + RESET);
            try {
                int code = new ProcessBuilder(List.of(cmd.trim().split("\\s+")))
                        .inheritIO().start().waitFor();
                System.out.println(code == 0
                        ? GREEN + "✓  Done" + RESET
                        : RED + "✗  Exited with status " + code + RESET);
            } catch (Exception e) {
                System.out.println(RED + "✗  Failed to run '" + cmd + "': " + e.getMessage() + RESET);
            }
        }

        System.out.print("\n" + YELLOW + "  Press Enter to return…" + RESET + "  ");
        System.out.flush();
        try {
            new BufferedReader(new InputStreamReader(System.in)).readLine();
        } catch (Exception ignored) {
            // best-effort — proceed and restore the TUI regardless
        }

        st.catalog.selectedIds.clear();
        st.installed.removeMode = false;
        if (wasRemoving) {
            st.installed.packages.clear();
            st.installed.names.clear();
        }
        st.requestFocus(PanelId.SECTIONS);
    }
}
