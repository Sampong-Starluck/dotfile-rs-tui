package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.config.AppProperties;
import com.sampong.dotfile.model.DotfileConfig;
import com.sampong.dotfile.model.ShellEntry;
import com.sampong.dotfile.model.ShellStatus;
import com.sampong.dotfile.service.AppCatalogService;
import com.sampong.dotfile.service.OsService;
import com.sampong.dotfile.service.PathService;
import com.sampong.dotfile.service.ScriptException;
import com.sampong.dotfile.service.ScriptService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.jspecify.annotations.Nullable;
import org.springframework.stereotype.Service;
import tools.jackson.databind.ObjectMapper;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.Set;

/** Port of Rust {@code service/script_service.rs}. */
@Slf4j
@Service
@RequiredArgsConstructor
public class ScriptServiceImp implements ScriptService {

    private static final Set<String> KNOWN_IDS = Set.of("bash", "zsh", "fish", "nushell", "powershell");

    private final PathService pathService;
    private final OsService osService;
    private final AppCatalogService appCatalogService;
    private final ObjectMapper mapper;
    private final AppProperties appProperties;

    /** Test-only override so {@code @TempDir} tests never touch the real home/APPDATA. */
    private @Nullable Path homeDirOverride;

    void overrideHomeDirForTest(Path dir) {
        this.homeDirOverride = dir;
    }

    // ── Constants / embedded scripts ────────────────────────────────────────────

    @Override
    public @Nullable String scriptContent(String shellId) {
        String resource = switch (shellId) {
            case "bash" -> "/scripts/bash/main_profile.sh";
            case "zsh" -> "/scripts/zsh/main_profile.zsh";
            case "fish" -> "/scripts/fish/main_profile.fish";
            case "nushell" -> "/scripts/nu/main_profile.nu";
            case "powershell" -> "/scripts/posh/main_profile.ps1";
            default -> null;
        };
        if (resource == null) {
            return null;
        }
        try (var in = getClass().getResourceAsStream(resource)) {
            if (in == null) {
                return null;
            }
            return new String(in.readAllBytes(), StandardCharsets.UTF_8);
        } catch (IOException e) {
            log.error("Failed to read script resource {}", resource, e);
            return null;
        }
    }

    @Override
    public boolean hasScript(String shellId) {
        return KNOWN_IDS.contains(shellId);
    }

    // ── Shell detection ──────────────────────────────────────────────────────────

    @Override
    public @Nullable String shellBinary(String shellId) {
        return switch (shellId) {
            case "bash" -> "bash";
            case "zsh" -> "zsh";
            case "fish" -> "fish";
            case "nushell" -> "nu";
            case "powershell" -> "pwsh";
            default -> null;
        };
    }

    @Override
    public boolean isShellDetected(String shellId) {
        String bin = shellBinary(shellId);
        return bin != null && pathService.isOnPath(bin);
    }

    // ── Paths ─────────────────────────────────────────────────────────────────────

    @Override
    public Path homeDir() {
        if (homeDirOverride != null) {
            return homeDirOverride;
        }
        String key = osService.isWindows() ? "USERPROFILE" : "HOME";
        String val = System.getenv(key);
        return val != null ? Path.of(val) : Path.of("/tmp");
    }

    @Override
    public Path scriptsBaseDir() {
        return baseDir(true).resolve("scripts");
    }

    @Override
    public Path configBaseDir() {
        return baseDir(false);
    }

    /** {@code withScripts} selects the data dir (Windows: {@code %APPDATA%}, Unix: {@code $XDG_DATA_HOME}) vs.
     *  the config dir (Windows: {@code %APPDATA%}, Unix: {@code $XDG_CONFIG_HOME}) — Windows uses the same
     *  root for both, so only the Unix branch actually differs by {@code withScripts}. */
    private Path baseDir(boolean withScripts) {
        if (osService.isWindows()) {
            String appData = System.getenv("APPDATA");
            Path base = appData != null ? Path.of(appData) : homeDir().resolve("AppData").resolve("Roaming");
            return base.resolve(appProperties.dataDirName());
        }
        String xdgVar = withScripts ? "XDG_DATA_HOME" : "XDG_CONFIG_HOME";
        String xdg = System.getenv(xdgVar);
        Path base = xdg != null ? Path.of(xdg) : homeDir().resolve(withScripts ? ".local/share" : ".config");
        return base.resolve(appProperties.dataDirName());
    }

    @Override
    public @Nullable Path scriptTarget(String shellId) {
        Path base = scriptsBaseDir();
        return switch (shellId) {
            case "bash" -> base.resolve("bash").resolve("main_profile.sh");
            case "zsh" -> base.resolve("zsh").resolve("main_profile.zsh");
            case "fish" -> base.resolve("fish").resolve("main_profile.fish");
            case "nushell" -> base.resolve("nu").resolve("main_profile.nu");
            case "powershell" -> base.resolve("posh").resolve("main_profile.ps1");
            default -> null;
        };
    }

    @Override
    public @Nullable Path shellProfilePath(String shellId) {
        return switch (shellId) {
            case "bash" -> (osService.isMac() || osService.isWindows())
                    ? homeDir().resolve(".bash_profile") : homeDir().resolve(".bashrc");
            case "zsh" -> homeDir().resolve(".zshrc");
            case "fish" -> osService.isWindows()
                    ? windowsAppData().resolve("fish").resolve("config.fish")
                    : xdgConfigHome().resolve("fish").resolve("config.fish");
            case "nushell" -> osService.isWindows()
                    ? windowsAppData().resolve("nushell").resolve("config.nu")
                    : xdgConfigHome().resolve("nushell").resolve("config.nu");
            case "powershell" -> osService.isWindows()
                    ? homeDir().resolve("Documents").resolve("PowerShell").resolve("Microsoft.PowerShell_profile.ps1")
                    : xdgConfigHome().resolve("powershell").resolve("Microsoft.PowerShell_profile.ps1");
            default -> null;
        };
    }

    private Path windowsAppData() {
        String appData = System.getenv("APPDATA");
        return appData != null ? Path.of(appData) : homeDir().resolve("AppData").resolve("Roaming");
    }

    private Path xdgConfigHome() {
        String xdg = System.getenv("XDG_CONFIG_HOME");
        return xdg != null ? Path.of(xdg) : homeDir().resolve(".config");
    }

    // ── Source line ───────────────────────────────────────────────────────────────

    @Override
    public @Nullable String sourceHint(String shellId) {
        Path target = scriptTarget(shellId);
        if (target == null) {
            return null;
        }
        String path = target.toString();
        return switch (shellId) {
            case "bash", "zsh", "fish", "nushell" -> "source \"" + path + "\"";
            case "powershell" -> ". \"" + path + "\"";
            default -> null;
        };
    }

    // ── Deploy / undeploy ─────────────────────────────────────────────────────────

    @Override
    public Path deployScript(String shellId) throws ScriptException {
        Path target = scriptTarget(shellId);
        if (target == null) {
            throw new ScriptException("no script for shell '" + shellId + "'");
        }
        String content = scriptContent(shellId);
        if (content == null) {
            throw new ScriptException("no script content for '" + shellId + "'");
        }
        try {
            if (target.getParent() != null) {
                Files.createDirectories(target.getParent());
            }
            Files.writeString(target, content, StandardCharsets.UTF_8);
        } catch (IOException e) {
            throw new ScriptException("write script: " + e.getMessage());
        }
        return target;
    }

    @Override
    public void undeployScript(String shellId) throws ScriptException {
        Path target = scriptTarget(shellId);
        if (target == null) {
            throw new ScriptException("no script for shell '" + shellId + "'");
        }
        if (Files.exists(target)) {
            try {
                Files.delete(target);
            } catch (IOException e) {
                throw new ScriptException("remove file: " + e.getMessage());
            }
        }
    }

    // ── Profile add / remove ──────────────────────────────────────────────────────

    @Override
    public ProfileResult addSourceToProfile(String shellId) throws ScriptException {
        Path profile = shellProfilePath(shellId);
        if (profile == null) {
            throw new ScriptException("no profile path for '" + shellId + "'");
        }
        String sourceLine = sourceHint(shellId);
        if (sourceLine == null) {
            throw new ScriptException("no source hint for '" + shellId + "'");
        }

        try {
            if (profile.getParent() != null) {
                Files.createDirectories(profile.getParent());
            }
        } catch (IOException e) {
            throw new ScriptException("create profile dir: " + e.getMessage());
        }

        String existing;
        try {
            existing = Files.exists(profile) ? Files.readString(profile, StandardCharsets.UTF_8) : "";
        } catch (IOException e) {
            throw new ScriptException("read profile: " + e.getMessage());
        }

        String trimmedSource = sourceLine.trim();
        if (existing.lines().anyMatch(l -> l.trim().equals(trimmedSource))) {
            return new ProfileResult(false, profile);
        }

        try {
            Files.writeString(profile, existing + "\n# dotfile-rs\n" + sourceLine + "\n",
                    StandardCharsets.UTF_8);
        } catch (IOException e) {
            throw new ScriptException("write profile: " + e.getMessage());
        }
        return new ProfileResult(true, profile);
    }

    @Override
    public ProfileResult removeSourceFromProfile(String shellId) throws ScriptException {
        Path profile = shellProfilePath(shellId);
        if (profile == null) {
            throw new ScriptException("no profile path for '" + shellId + "'");
        }
        if (!Files.exists(profile)) {
            return new ProfileResult(false, profile);
        }
        String sourceLine = sourceHint(shellId);
        if (sourceLine == null) {
            throw new ScriptException("no source hint for '" + shellId + "'");
        }

        String content;
        try {
            content = Files.readString(profile, StandardCharsets.UTF_8);
        } catch (IOException e) {
            throw new ScriptException("read profile: " + e.getMessage());
        }

        String trimmedSource = sourceLine.trim();
        if (content.lines().noneMatch(l -> l.trim().equals(trimmedSource))) {
            return new ProfileResult(false, profile);
        }

        String cleaned = stripSourceBlock(content, trimmedSource);
        try {
            Files.writeString(profile, cleaned, StandardCharsets.UTF_8);
        } catch (IOException e) {
            throw new ScriptException("write profile: " + e.getMessage());
        }
        return new ProfileResult(true, profile);
    }

    /** Port of {@code strip_source_block}: removes the exact 3-line {@code [blank, "# dotfile-rs",
     *  sourceLine]} block plus any stray bare source line; preserves a trailing newline iff the
     *  original content had one. */
    private static String stripSourceBlock(String content, String trimmedSource) {
        List<String> lines = content.lines().toList();
        List<String> out = new ArrayList<>(lines.size());
        int i = 0;
        while (i < lines.size()) {
            if (i + 2 < lines.size()
                    && lines.get(i).trim().isEmpty()
                    && lines.get(i + 1).trim().equals("# dotfile-rs")
                    && lines.get(i + 2).trim().equals(trimmedSource)) {
                i += 3;
                continue;
            }
            if (lines.get(i).trim().equals(trimmedSource)) {
                i += 1;
                continue;
            }
            out.add(lines.get(i));
            i += 1;
        }
        String result = String.join("\n", out);
        return content.endsWith("\n") ? result + "\n" : result;
    }

    // ── Primary-shell config ──────────────────────────────────────────────────────

    @Override
    public Path configPath() {
        return configBaseDir().resolve("config.json");
    }

    @Override
    public DotfileConfig readConfig() {
        Path path = configPath();
        if (!Files.exists(path)) {
            return new DotfileConfig(null);
        }
        try {
            return mapper.readValue(Files.readString(path, StandardCharsets.UTF_8), DotfileConfig.class);
        } catch (Exception e) {
            log.debug("Failed to read/parse {}, using defaults", path, e);
            return new DotfileConfig(null);
        }
    }

    @Override
    public void writeConfig(DotfileConfig config) throws ScriptException {
        Path path = configPath();
        try {
            if (path.getParent() != null) {
                Files.createDirectories(path.getParent());
            }
            Files.writeString(path, mapper.writeValueAsString(config), StandardCharsets.UTF_8);
        } catch (Exception e) {
            throw new ScriptException("write config: " + e.getMessage());
        }
    }

    @Override
    public void setPrimaryShell(String shellId) throws ScriptException {
        writeConfig(new DotfileConfig(shellId));
    }

    @Override
    public void clearPrimaryShell() throws ScriptException {
        writeConfig(new DotfileConfig(null));
    }

    @Override
    public Optional<String> detectDefaultShell() {
        if (osService.isWindows()) {
            return pathService.isOnPath("pwsh") ? Optional.of("powershell") : Optional.empty();
        }
        String shellEnv = System.getenv("SHELL");
        if (shellEnv == null) {
            return Optional.empty();
        }
        int slash = shellEnv.lastIndexOf('/');
        String name = slash >= 0 ? shellEnv.substring(slash + 1) : shellEnv;
        return switch (name) {
            case "bash" -> Optional.of("bash");
            case "zsh" -> Optional.of("zsh");
            case "fish" -> Optional.of("fish");
            case "nu" -> Optional.of("nushell");
            case "pwsh", "powershell" -> Optional.of("powershell");
            default -> Optional.empty();
        };
    }

    @Override
    public Optional<String> effectivePrimaryShell() {
        String configured = readConfig().primaryShell();
        return configured != null ? Optional.of(configured) : detectDefaultShell();
    }

    @Override
    public Optional<String> chshCommand(String shellId) {
        if (osService.isWindows()) {
            return Optional.empty();
        }
        String bin = shellBinary(shellId);
        if (bin == null) {
            return Optional.empty();
        }
        return pathService.which(bin).map(path -> "chsh -s " + path);
    }

    // ── Status assembly ──────────────────────────────────────────────────────────

    @Override
    public List<ShellStatus> loadShellStatuses() {
        List<ShellStatus> out = new ArrayList<>();
        for (ShellEntry entry : appCatalogService.readShellsJson()) {
            String id = entry.id();
            boolean detected = isShellDetected(id);
            Path target = scriptTarget(id);
            boolean deployed = target != null && Files.exists(target);
            out.add(new ShellStatus(entry, detected, deployed, target,
                    shellBinary(id), shellProfilePath(id), sourceHint(id)));
        }
        return out;
    }
}
