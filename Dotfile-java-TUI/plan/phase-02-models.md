# Phase 2 — Domain models + JSON loading

**Goal:** every data type the app needs exists as a Java 25 record/enum, and
`apps.json` / `shells.json` load from the classpath via the Spring-managed
Jackson `ObjectMapper`. No `dev.tamboui` imports anywhere in this phase.

Rust behavior references: `../src/models/*.rs`. UI enums follow the
**lazygit design in PLAN.md §5**, not the Rust tab/focus enums.

Files go in `io.github.sampongstarluck.dotfile.model` unless noted.

> Spring Boot 4.1 ships **Jackson 3** — the `ObjectMapper` you inject may live
> in `tools.jackson.databind` (annotations still `com.fasterxml.jackson.annotation`).
> Always inject the Boot-managed mapper; fix imports to whatever autowires.

## 2.1 UI enums (lazygit model — replaces Rust `TabModel`/`AppFocus`)

```java
/** Side panels + main, in Tab-cycle order. Number keys 1–4 map to the first four. */
public enum PanelId {
    STATUS, MANAGERS, SECTIONS, SHELLS, MAIN;

    public PanelId next() { return values()[(ordinal() + 1) % values().length]; }
    public PanelId prev() { return values()[(ordinal() + values().length - 1) % values().length]; }
}

/** What the main (right) panel is showing. */
public enum MainView { COMMANDS, APPS, INSTALLED, SEARCH_RESULTS, SHELL_INFO }
```

(The sealed `Popup` hierarchy is UI-state, defined in `base/` during Phase 5.)

## 2.2 Platform enums

### `LinuxDistro.java`

```java
public enum LinuxDistro {
    DEBIAN("Debian / Ubuntu"), FEDORA("Fedora / RHEL"), ARCH("Arch Linux"),
    VOID("Void Linux"), OTHER("Other");

    private final String label;
    LinuxDistro(String label) { this.label = label; }
    public String label() { return label; }
}
```

### `OperatingSystem.java`

```java
public record OperatingSystem(Kind kind, LinuxDistro distro, String distroName) {
    public enum Kind { WINDOWS, MACOS, LINUX, UNKNOWN }

    public String label() {
        return switch (kind) {
            case WINDOWS -> "Windows"; case MACOS -> "macOS";
            case LINUX -> "Linux";     case UNKNOWN -> "Unknown";
        };
    }

    /** "Linux (Arch Linux)" — port of the Rust Display impl */
    @Override public String toString() {
        if (kind == Kind.LINUX && distro != null) {
            String d = (distro == LinuxDistro.OTHER && distroName != null) ? distroName : distro.label();
            return "Linux (" + d + ")";
        }
        return label();
    }
}
```

### `PackageManager.java` (data only — detection is Phase 3)

Port `binary()`, `label()`, `description()`, and the full `commands()` tables
**verbatim** from `../src/models/package_manager.rs:26-143` and the
description table from `../src/ui/features/app_tab.rs:1888`.

```java
public enum PackageManager {
    WINGET("winget", "winget"),   SCOOP("scoop", "scoop"),   CHOCO("choco", "choco"),
    APT("apt", "apt"),            DNF("dnf", "dnf"),         PACMAN("pacman", "pacman"),
    YAY("yay", "yay (AUR)"),      PARU("paru", "paru (AUR)"),
    XBPS("xbps-install", "xbps"), BREW("brew", "Homebrew");

    private final String binary;
    private final String label;
    PackageManager(String binary, String label) { this.binary = binary; this.label = label; }
    public String binary() { return binary; }
    public String label()  { return label; }

    public List<PmCommand> commands() {
        return switch (this) {
            case APT -> List.of(
                new PmCommand("install", "apt install <pkg>",    "Install a package"),
                new PmCommand("remove",  "apt remove <pkg>",     "Remove a package"),
                new PmCommand("update",  "apt update",           "Refresh package index"),
                new PmCommand("upgrade", "apt upgrade",          "Upgrade all packages"),
                new PmCommand("search",  "apt search <pkg>",     "Search available packages"),
                new PmCommand("show",    "apt show <pkg>",       "Show package details"),
                new PmCommand("list",    "apt list --installed", "List installed packages"));
            // COPY the remaining 9 cases (DNF, PACMAN, YAY, PARU, XBPS, WINGET,
            // SCOOP, CHOCO, BREW) from the Rust file — identical strings.
        };
    }

    public String description() {
        return switch (this) {
            case WINGET -> "Windows built-in, largest catalog";
            case SCOOP  -> "portable installs, no admin needed";
            case CHOCO  -> "traditional installs, wide support";
            case APT    -> "Debian/Ubuntu";
            case DNF    -> "Red Hat/Fedora";
            case PACMAN -> "Arch Linux default package manager";
            case YAY    -> "Yay (Yet Another Yogurt) AUR helper";
            case PARU   -> "Paru — feature-rich AUR helper";
            case XBPS   -> "Void Linux";
            case BREW   -> "";
        };
    }
}
```

## 2.3 Plain records

```java
public record PmCommand(String name, String command, String description) {}

public record SearchResult(String name, String id, String version) {}
```

### Catalog records (port of `apps.rs`)

```java
public record AppEntry(
        String name,
        String id,
        Map<String, Map<String, String>> platforms) {}  // platforms.get("windows").get("winget") → pkg id

public record AppSection(String section, List<AppEntry> apps) {}
// apps.json root is a JSON array → List<AppSection>
```

### Shell records

`shells.json` carries extra fields (`function`, `version`, `lastUpdated`) —
ignore unknowns:

```java
@JsonIgnoreProperties(ignoreUnknown = true)
public record ShellEntry(
        String id, String name, boolean hidden, String description,
        int order, List<String> platforms, List<String> requires) {
    public ShellEntry {
        if (description == null) description = "";
        if (platforms == null) platforms = List.of();
        if (requires == null) requires = List.of();
    }
}

@JsonIgnoreProperties(ignoreUnknown = true)
public record ShellsFile(List<ShellEntry> shells) {}
```

### `ShellStatus.java` and `DotfileConfig.java`

```java
public record ShellStatus(ShellEntry entry, boolean detected, boolean deployed, Path targetPath) {}

@JsonIgnoreProperties(ignoreUnknown = true)
public record DotfileConfig(@JsonProperty("primary_shell") String primaryShell) {}
// snake_case on disk = byte-compatible with the Rust app's config.json
```

## 2.4 Catalog loading service

Service convention (PLAN.md §4): the **interface** lives in `service/`, the
**implementation** in `service/implementation/` with the `Imp` suffix and
the `@Service` annotation. First instance:

`service/AppCatalogService.java`:

```java
public interface AppCatalogService {
    List<AppSection> readAppsJson();
    List<AppSection> filterByPlatform(List<AppSection> apps, String osKey, String detectedMgr);
    List<ShellEntry> readShellsJson();
}
```

`service/implementation/AppCatalogServiceImp.java` (port of `app_service.rs`
+ `script_service.rs::read_shells`):

```java
@Slf4j
@Service
@RequiredArgsConstructor
public class AppCatalogServiceImp implements AppCatalogService {
    private final ObjectMapper mapper;               // Boot-managed (Jackson 3)

    public List<AppSection> readAppsJson() {
        try (InputStream in = getClass().getResourceAsStream("/data/apps.json")) {
            return mapper.readValue(in, new TypeReference<List<AppSection>>() {});
        } catch (Exception e) {
            log.error("Failed to load apps.json", e);
            return List.of();
        }
    }

    /** keep entries whose platforms[osKey] contains detectedMgr; drop empty sections */
    public List<AppSection> filterByPlatform(List<AppSection> apps, String osKey, String detectedMgr) {
        List<AppSection> out = new ArrayList<>();
        for (AppSection section : apps) {
            List<AppEntry> keep = section.apps().stream()
                .filter(e -> {
                    Map<String, String> platform = e.platforms().get(osKey);
                    return platform != null && platform.containsKey(detectedMgr);
                })
                .toList();
            if (!keep.isEmpty()) out.add(new AppSection(section.section(), keep));
        }
        return out;
    }

    /** shells.json → visible, known-script shells sorted by order */
    public List<ShellEntry> readShellsJson() {
        // read /data/shells.json → ShellsFile; filter !hidden and
        // id ∈ {bash, zsh, fish, nushell, powershell}; sort by order
    }
}
```

## 2.5 Unit tests

1. `AppCatalogServiceTest`
   - `readAppsJson()` non-empty; first section `"Terminal and Shells"`;
     `"PowerShell 7"` → `platforms.windows.winget == "Microsoft.PowerShell"`.
   - `filterByPlatform(apps, "windows", "winget")` drops `Zsh`.
   - `readShellsJson()` → 5 entries, sorted by order, powershell first.
2. `PanelIdTest` — full `next()`/`prev()` cycle both directions.
3. `PackageManagerTest` — every constant has non-empty `commands()`;
   `XBPS.binary().equals("xbps-install")`.
4. `DotfileConfigTest` — serializes to `{"primary_shell":"zsh"}` and reads it back.

## Definition of Done (Phase 2)

- [x] All models compile; zero `dev.tamboui` imports in `model/` + `service/`
- [x] `AppCatalogService` is an interface; `AppCatalogServiceImp` in `service/implementation/` is the only `@Service`; tests inject the interface
- [x] `mise run test` green with all §2.5 tests
- [x] Jackson imports resolved against what Boot 4.1 actually provides (note the package here: `tools.jackson.databind.ObjectMapper` / `tools.jackson.core.type.TypeReference`, group `tools.jackson.core`, pulled in via the `spring-boot-starter-jackson` starter added to `pom.xml`; annotations stayed on `com.fasterxml.jackson.annotation`)
