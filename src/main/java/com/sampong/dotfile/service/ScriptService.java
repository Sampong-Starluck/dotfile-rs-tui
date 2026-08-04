package com.sampong.dotfile.service;

import com.sampong.dotfile.model.DotfileConfig;
import com.sampong.dotfile.model.ShellStatus;
import org.jspecify.annotations.Nullable;

import java.nio.file.Path;
import java.util.List;
import java.util.Optional;

/** Port of Rust {@code service/script_service.rs} — shell detection, deploy/undeploy, profile edits, primary-shell config. */
public interface ScriptService {

    /** {@code (changed, profile)} — mirrors the Rust {@code Result<(bool, PathBuf), String>} pair. */
    record ProfileResult(boolean changed, Path profile) {
    }

    @Nullable String scriptContent(String shellId);

    boolean hasScript(String shellId);

    @Nullable String shellBinary(String shellId);

    boolean isShellDetected(String shellId);

    Path homeDir();

    Path scriptsBaseDir();

    Path configBaseDir();

    @Nullable Path scriptTarget(String shellId);

    @Nullable Path shellProfilePath(String shellId);

    @Nullable String sourceHint(String shellId);

    Path deployScript(String shellId) throws ScriptException;

    void undeployScript(String shellId) throws ScriptException;

    ProfileResult addSourceToProfile(String shellId) throws ScriptException;

    ProfileResult removeSourceFromProfile(String shellId) throws ScriptException;

    Path configPath();

    DotfileConfig readConfig();

    void writeConfig(DotfileConfig config) throws ScriptException;

    void setPrimaryShell(String shellId) throws ScriptException;

    void clearPrimaryShell() throws ScriptException;

    Optional<String> detectDefaultShell();

    Optional<String> effectivePrimaryShell();

    /** {@code chsh -s <path>} command, or empty on Windows / when the binary isn't on PATH. */
    Optional<String> chshCommand(String shellId);

    List<ShellStatus> loadShellStatuses();
}
