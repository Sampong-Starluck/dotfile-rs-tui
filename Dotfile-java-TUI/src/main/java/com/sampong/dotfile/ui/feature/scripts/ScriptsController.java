package com.sampong.dotfile.ui.feature.scripts;

import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.tui.event.KeyEvent;
import com.sampong.dotfile.base.KeyController;
import com.sampong.dotfile.model.ShellStatus;
import com.sampong.dotfile.service.ScriptException;
import com.sampong.dotfile.service.ScriptService;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.state.AppState;
import org.jspecify.annotations.Nullable;

import java.util.List;
import java.util.Optional;

/** Keys while {@code [4]-Shells} is focused: navigate + deploy/undeploy/primary-shell actions.
 *  Port of {@code script_tab.rs::handle_key} + its {@code *_selected} action functions. */
public class ScriptsController implements KeyController {

    private final ScriptService scriptService;

    public ScriptsController(ScriptService scriptService) {
        this.scriptService = scriptService;
    }

    @Override
    public EventResult handleKey(KeyEvent key, AppState st) {
        List<ShellStatus> shells = st.scripts.shells;
        int count = shells != null ? shells.size() : 0;

        if (Keys.isDown(key)) {
            if (count > 0) {
                st.scripts.shellCursor = Math.min(st.scripts.shellCursor + 1, count - 1);
            }
            return EventResult.HANDLED;
        }
        if (Keys.isUp(key)) {
            st.scripts.shellCursor = Math.max(0, st.scripts.shellCursor - 1);
            return EventResult.HANDLED;
        }
        if (Keys.isEnter(key)) {
            deploy(st);
            return EventResult.HANDLED;
        }
        if (key.isChar('d')) {
            undeploy(st);
            return EventResult.HANDLED;
        }
        if (key.isChar('p')) {
            setPrimary(st);
            return EventResult.HANDLED;
        }
        if (key.isChar('c')) {
            clearPrimary(st);
            return EventResult.HANDLED;
        }
        if (key.isChar('r')) {
            st.scripts.shells = null;
            st.scripts.log.add("  Refreshed shell status.");
            return EventResult.HANDLED;
        }
        return EventResult.UNHANDLED;
    }

    private static @Nullable ShellStatus selected(AppState st) {
        List<ShellStatus> shells = st.scripts.shells;
        if (shells == null || st.scripts.shellCursor >= shells.size()) {
            return null;
        }
        return shells.get(st.scripts.shellCursor);
    }

    private void deploy(AppState st) {
        ShellStatus status = selected(st);
        if (status == null) {
            return;
        }
        String id = status.entry().id();

        try {
            var path = scriptService.deployScript(id);
            st.scripts.log.add("  ✓ Script → " + path);
        } catch (ScriptException e) {
            st.scripts.log.add("  ✗ Deploy " + status.entry().name() + " failed: " + e.getMessage());
            st.scripts.shells = null;
            return;
        }

        try {
            var result = scriptService.addSourceToProfile(id);
            st.scripts.log.add(result.changed()
                    ? "  ✓ Added source line → " + result.profile()
                    : "  — Already in " + result.profile());
        } catch (ScriptException e) {
            st.scripts.log.add("  ✗ Profile update failed: " + e.getMessage());
        }

        st.scripts.shells = null;
    }

    private void undeploy(AppState st) {
        ShellStatus status = selected(st);
        if (status == null) {
            return;
        }
        if (!status.deployed()) {
            st.scripts.log.add("  — " + status.entry().name() + " is not deployed.");
            return;
        }
        String id = status.entry().id();

        try {
            scriptService.undeployScript(id);
            st.scripts.log.add("  ✓ Removed " + status.entry().name() + " script.");
        } catch (ScriptException e) {
            st.scripts.log.add("  ✗ Remove " + status.entry().name() + " failed: " + e.getMessage());
            st.scripts.shells = null;
            return;
        }

        try {
            var result = scriptService.removeSourceFromProfile(id);
            st.scripts.log.add(result.changed()
                    ? "  ✓ Removed source line from " + result.profile()
                    : "  — Source line not found in profile.");
        } catch (ScriptException e) {
            st.scripts.log.add("  ✗ Profile cleanup failed: " + e.getMessage());
        }

        st.scripts.shells = null;
    }

    private void setPrimary(AppState st) {
        ShellStatus status = selected(st);
        if (status == null) {
            return;
        }
        String id = status.entry().id();

        try {
            scriptService.setPrimaryShell(id);
            st.scripts.primaryShell = id;
            st.scripts.explicitPrimaryShell = id;
            st.scripts.log.add("  ★  " + status.entry().name() + " set as primary shell.");
            Optional<String> chsh = scriptService.chshCommand(id);
            if (chsh.isPresent()) {
                st.scripts.log.add("  ▶  Running: " + chsh.get());
                st.install.runExternal.add(chsh.get());
            } else {
                st.scripts.log.add("  — chsh not available on this platform.");
            }
        } catch (ScriptException e) {
            st.scripts.log.add("  ✗ Set primary failed: " + e.getMessage());
        }
    }

    private void clearPrimary(AppState st) {
        try {
            scriptService.clearPrimaryShell();
            st.scripts.explicitPrimaryShell = null;
            Optional<String> detected = scriptService.detectDefaultShell();
            st.scripts.primaryShell = detected.orElse(null);
            st.scripts.log.add(detected.isPresent()
                    ? "  ◇  Primary cleared. System default: " + detected.get() + "."
                    : "  ◇  Primary cleared. No system default detected.");
        } catch (ScriptException e) {
            st.scripts.log.add("  ✗ Clear primary failed: " + e.getMessage());
        }
    }
}
