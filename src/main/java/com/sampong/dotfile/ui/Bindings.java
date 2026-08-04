package com.sampong.dotfile.ui;

import com.sampong.dotfile.base.ConfirmActionPopup;
import com.sampong.dotfile.base.CustomInputPopup;
import com.sampong.dotfile.base.HelpPopup;
import com.sampong.dotfile.base.InstallLogPopup;
import com.sampong.dotfile.base.SearchInputPopup;
import com.sampong.dotfile.base.SudoPopup;
import com.sampong.dotfile.model.MainView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.ui.state.AppState;

import java.util.ArrayList;
import java.util.List;

/**
 * Single source of truth for every keybinding hint (PLAN.md phase-10 §10.1). {@link #forState}
 * drives the bottom {@code HintBar}; {@link #sections()} drives {@code HelpPopup} — nothing else
 * may render key-hint text (DRY).
 */
public final class Bindings {
    private Bindings() {
    }

    public record Binding(String key, String desc) {
    }

    public record Section(String label, List<Binding> bindings) {
    }

    private static final List<Binding> GLOBAL = List.of(
            new Binding("1-4", "jump"),
            new Binding("tab", "cycle"),
            new Binding("?", "help"),
            new Binding("q", "quit"));

    private static final List<Binding> CTX_HELP_POPUP = List.of(
            new Binding("j/k", "scroll"), new Binding("pgup/pgdn", "page"), new Binding("esc/?", "close"));
    private static final List<Binding> CTX_TEXT_INPUT_POPUP = List.of(
            new Binding("enter", "confirm"), new Binding("esc", "cancel"));
    private static final List<Binding> CTX_CONFIRM_POPUP = List.of(
            new Binding("y", "yes"), new Binding("n", "no"));
    private static final List<Binding> CTX_SUDO_POPUP = List.of(
            new Binding("enter", "run"), new Binding("esc", "cancel"));
    private static final List<Binding> CTX_INSTALL_LOG_POPUP = List.of(
            new Binding("type+enter", "respond"), new Binding("esc", "close"));
    private static final List<Binding> CTX_MANAGERS_STATUS = List.of(
            new Binding("enter", "use manager"), new Binding("j/k", "move"), new Binding("pgup/pgdn", "scroll"));
    private static final List<Binding> CTX_SECTIONS = List.of(
            new Binding("enter", "open"), new Binding("/", "search"), new Binding("l", "installed"),
            new Binding("c", "custom"), new Binding("d", "install"));
    private static final List<Binding> CTX_MAIN_APPS = List.of(
            new Binding("space", "select"), new Binding("d", "install"), new Binding("/", "search"),
            new Binding("l", "installed"), new Binding("esc", "back"));
    private static final List<Binding> CTX_MAIN_SEARCH_RESULTS = List.of(
            new Binding("space", "select"), new Binding("d", "install"), new Binding("/", "edit query"),
            new Binding("esc", "back"));
    private static final List<Binding> CTX_MAIN_INSTALLED = List.of(
            new Binding("space", "select"), new Binding("d", "remove"), new Binding("u", "update"),
            new Binding("r", "refresh"), new Binding("/", "filter"), new Binding("esc", "back"));
    private static final List<Binding> CTX_MAIN_INSTALLED_FILTERING = List.of(
            new Binding("type", "filter"), new Binding("j/k", "move"),
            new Binding("enter", "done"), new Binding("esc", "clear"));
    private static final List<Binding> CTX_SHELLS = List.of(
            new Binding("enter", "deploy"), new Binding("d", "undeploy"), new Binding("p", "primary"),
            new Binding("c", "clear"), new Binding("r", "refresh"));

    /** Context bindings for the current state with the global suffix always appended. */
    public static List<Binding> forState(AppState st) {
        List<Binding> ctx = context(st);
        List<Binding> all = new ArrayList<>(ctx.size() + GLOBAL.size());
        all.addAll(ctx);
        all.addAll(GLOBAL);
        return all;
    }

    /** Every context row, grouped for the Help popup — generated from the same tables as {@link #forState}. */
    public static List<Section> sections() {
        return List.of(
                new Section("Global", GLOBAL),
                new Section("Help", CTX_HELP_POPUP),
                new Section("Search / custom input", CTX_TEXT_INPUT_POPUP),
                new Section("Confirm", CTX_CONFIRM_POPUP),
                new Section("Sudo", CTX_SUDO_POPUP),
                new Section("Install log", CTX_INSTALL_LOG_POPUP),
                new Section("Managers / Status", CTX_MANAGERS_STATUS),
                new Section("Sections", CTX_SECTIONS),
                new Section("Main · Apps", CTX_MAIN_APPS),
                new Section("Main · Search results", CTX_MAIN_SEARCH_RESULTS),
                new Section("Main · Installed", CTX_MAIN_INSTALLED),
                new Section("Main · Installed (filtering)", CTX_MAIN_INSTALLED_FILTERING),
                new Section("Shells", CTX_SHELLS));
    }

    private static List<Binding> context(AppState st) {
        if (st.popup != null) {
            return switch (st.popup) {
                case HelpPopup _ -> CTX_HELP_POPUP;
                case SearchInputPopup _, CustomInputPopup _ -> CTX_TEXT_INPUT_POPUP;
                case ConfirmActionPopup _ -> CTX_CONFIRM_POPUP;
                case SudoPopup _ -> CTX_SUDO_POPUP;
                case InstallLogPopup _ -> CTX_INSTALL_LOG_POPUP;
            };
        }
        if (st.focused == PanelId.MAIN) {
            if (st.mainView == MainView.INSTALLED && st.installed.filtering) {
                return CTX_MAIN_INSTALLED_FILTERING;
            }
            return switch (st.mainView) {
                case APPS -> CTX_MAIN_APPS;
                case SEARCH_RESULTS -> CTX_MAIN_SEARCH_RESULTS;
                case INSTALLED -> CTX_MAIN_INSTALLED;
                case COMMANDS, SHELL_INFO -> List.of();
            };
        }
        return switch (st.focused) {
            case STATUS, MANAGERS -> CTX_MANAGERS_STATUS;
            case SECTIONS -> CTX_SECTIONS;
            case SHELLS -> CTX_SHELLS;
            case MAIN -> List.of();
        };
    }
}
