# Phase 3 — Platform services: OS detect, `which`, PM detect, command builders

**Goal:** all environment-probing and command-string logic from the Rust
`src/models/os.rs`, `src/models/package_manager.rs` (detection half),
`src/service/system_service.rs`, `src/service/install_service.rs`, and parts
of `src/service/search_service.rs`. Pure Java, fully unit-testable.

**Convention (PLAN.md §4) applies to every service in this phase:** define
the interface in `…dotfile.service` exposing exactly the public methods
shown below; put the implementation in `…dotfile.service.implementation` as
`<Name>Imp implements <Name>` with the `@Service` annotation. The code
snippets below show the implementation bodies — split them accordingly.
(`OsService`/`OsServiceImp`, `PathService`/`PathServiceImp`,
`PackageManagerService`/`PackageManagerServiceImp`,
`SystemService`/`SystemServiceImp`,
`InstallCommandService`/`InstallCommandServiceImp`,
`SearchService`/`SearchServiceImp`. The `SearchService.Cmd` record and other
supporting types belong on the interface.) Every Imp class:
`@Slf4j @Service @RequiredArgsConstructor` with `private final` interface
deps — PLAN.md §4a.

## 3.1 `OsService` (port of `os.rs`)

```java
@Service
public class OsService {

    public boolean isWindows() { return osName().contains("win"); }
    public boolean isMac()     { return osName().contains("mac"); }
    public boolean isLinux()   { return osName().contains("linux") || osName().contains("nix"); }
    private String osName()    { return System.getProperty("os.name", "").toLowerCase(); }

    /** "windows" | "macos" | "linux" — the key used in apps.json platforms maps */
    public String osKey() {
        if (isWindows()) return "windows";
        if (isMac())     return "macos";
        return "linux";
    }

    public OperatingSystem detect() {
        if (isWindows()) return new OperatingSystem(Kind.WINDOWS, null, null);
        if (isMac())     return new OperatingSystem(Kind.MACOS, null, null);
        if (isLinux()) {
            var d = detectDistro();                    // may be null
            return new OperatingSystem(Kind.LINUX, d.distro(), d.name());
        }
        return new OperatingSystem(Kind.UNKNOWN, null, null);
    }

    /** Port of LinuxDistro::detect() — read /etc/os-release, check ID= then ID_LIKE= */
    // parse lines starting with "ID=" and "ID_LIKE=", strip quotes, lowercase,
    // concatenate both; contains("arch")→ARCH, ("fedora"|"rhel")→FEDORA,
    // ("debian"|"ubuntu")→DEBIAN, ("void")→VOID, else OTHER with the raw ID value.
    // Return a small record DistroResult(LinuxDistro distro, String name).
    // On Windows this file doesn't exist → guard with Files.exists().
}
```

## 3.2 `PathService` — the `which` crate replacement

Java has no `which`; implement it once, correctly, including Windows
`PATHEXT` handling (so `scoop` resolves to `scoop.cmd`, `winget` to
`winget.exe`):

```java
@Service
public class PathService {

    /** Returns the full path of an executable on PATH, or empty. Port of which::which(). */
    public Optional<Path> which(String binary) {
        String pathEnv = System.getenv("PATH");
        if (pathEnv == null) return Optional.empty();

        boolean windows = System.getProperty("os.name", "").toLowerCase().contains("win");
        List<String> extensions = new ArrayList<>();
        extensions.add("");                                   // exact name first
        if (windows) {
            String pathext = System.getenv("PATHEXT");        // ".COM;.EXE;.BAT;.CMD;..."
            if (pathext == null) pathext = ".COM;.EXE;.BAT;.CMD;.PS1";
            for (String ext : pathext.split(";")) {
                if (!ext.isBlank()) extensions.add(ext.toLowerCase());
            }
        }

        for (String dir : pathEnv.split(File.pathSeparator)) {
            if (dir.isBlank()) continue;
            for (String ext : extensions) {
                Path candidate = Path.of(dir, binary + ext);
                if (Files.isRegularFile(candidate)
                        && (windows || Files.isExecutable(candidate))) {
                    return Optional.of(candidate);
                }
            }
        }
        return Optional.empty();
    }

    public boolean isOnPath(String binary) { return which(binary).isPresent(); }
}
```

## 3.3 `PackageManagerService` (detection half of `package_manager.rs`)

```java
@Service
public class PackageManagerService {
    private final OsService os;
    private final PathService path;
    // constructor injection

    /** Port of PackageManager::detect() — OS-filtered candidates, then PATH check */
    public List<PackageManager> detect() {
        return candidatesFor(os.detect()).stream()
                .filter(pm -> path.isOnPath(pm.binary()))
                .toList();
    }

    /** Port of candidates_for() — copy the exact mapping from package_manager.rs:174 */
    List<PackageManager> candidatesFor(OperatingSystem o) {
        return switch (o.kind()) {
            case WINDOWS -> List.of(WINGET, SCOOP, CHOCO);
            case MACOS   -> List.of(BREW);
            case LINUX   -> {
                if (o.distro() == null) yield List.of(APT, DNF, PACMAN, YAY, XBPS);
                yield switch (o.distro()) {
                    case ARCH   -> List.of(PACMAN, YAY, PARU);
                    case DEBIAN -> List.of(APT);
                    case FEDORA -> List.of(DNF);
                    case VOID   -> List.of(XBPS);
                    case OTHER  -> List.of(APT, DNF, PACMAN, YAY, XBPS);
                };
            }
            default -> List.of();
        };
    }
}
```

## 3.4 `SystemService` (port of `system_service.rs` — 3 tiny functions)

```java
@Service
public class SystemService {

    /** Rust: unsafe { libc::getuid() == 0 }; Windows always false */
    public boolean isRoot() {
        if (System.getProperty("os.name", "").toLowerCase().contains("win")) return false;
        return "root".equals(System.getProperty("user.name"));
    }

    public boolean requiresSudo(String mgr) {
        return switch (mgr) {
            case "pacman", "apt", "apt-get", "dnf", "yum", "xbps-install", "apk" -> true;
            default -> false;
        };
    }

    /** managers whose prompts go to the real terminal → must suspend the TUI */
    public boolean requiresInteractive(String mgr) {
        return switch (mgr) {
            case "pacman", "apt", "apt-get", "dnf", "yum", "xbps-install", "apk",
                 "yay", "paru", "choco" -> true;
            default -> false;
        };
    }
}
```

## 3.5 `InstallCommandService` (port of `install_service.rs`)

Static-style methods; copy the two `switch` tables **verbatim** from
`../src/service/install_service.rs` (`install_command`, `remove_command` —
13 cases each plus default). Plus the fallback resolution:

```java
@Service
public class InstallCommandService {
    private final OsService os;

    /** Port of get_install_command(): resolve entry's package id for this manager,
     *  falling back to the OS's preferred manager order when detectedMgr is absent. */
    public Optional<String> installCommandFor(AppEntry entry, String detectedMgr) {
        Map<String, String> platform = entry.platforms().get(os.osKey());
        if (platform == null) return Optional.empty();

        String mgr = detectedMgr;
        if (!platform.containsKey(mgr)) {
            List<String> fallbacks = switch (os.osKey()) {
                case "windows" -> List.of("winget", "scoop", "choco");
                case "macos"   -> List.of("brew");
                default        -> List.of("pacman", "yay", "apt", "dnf", "xbps-install");
            };
            mgr = fallbacks.stream().filter(platform::containsKey).findFirst().orElse(null);
            if (mgr == null) return Optional.empty();
        }
        return Optional.of(installCommand(mgr, platform.get(mgr)));
    }

    public String installCommand(String mgr, String pkg) { /* copy switch verbatim */ }
    public String removeCommand(String mgr, String pkg)  { /* copy switch verbatim */ }
}
```

## 3.6 `SearchService` — command tables only (parsers come in Phase 4)

Port from `../src/service/search_service.rs` these three pure functions
(copy every case exactly):

```java
@Service
public class SearchService {
    /** Port of search_command(mgr, query) → (binary, args). Model as: */
    public record Cmd(String binary, List<String> args) {}

    public Cmd searchCommand(String mgr, String query) { /* verbatim table, search_service.rs:12 */ }
    public Cmd listCommand(String mgr)                 { /* verbatim table, search_service.rs:53 */ }
    public String searchHint(String mgr)               { /* verbatim table, search_service.rs:85 */ }
}
```

## 3.7 Unit tests

- `PathServiceTest` — on Windows: `which("cmd")` is present and ends with
  `cmd.exe` (case-insensitive); `which("definitely-not-a-binary-xyz")` empty.
- `PackageManagerServiceTest` — `candidatesFor(WINDOWS)` == `[WINGET, SCOOP, CHOCO]`;
  Arch → `[PACMAN, YAY, PARU]`; unknown-distro Linux → 5 candidates.
- `SystemServiceTest` — `requiresSudo("pacman")` true, `("winget")` false;
  `requiresInteractive("choco")` true, `("brew")` false, `("scoop")` false.
- `InstallCommandServiceTest` — `installCommand("winget", "Git.Git")` ==
  `"winget install --id Git.Git -e"`; `removeCommand("choco","git")` ==
  `"choco uninstall git -y"`; fallback test: entry with only
  `{windows:{scoop: "git"}}` + detectedMgr `winget` → `"scoop install git"`.
- `SearchServiceTest` — `searchCommand("choco","git")` ==
  `("choco", ["search","git","--limit-output"])`;
  `listCommand("xbps-install")` == `("xbps-query", ["-l"])`.

## Definition of Done (Phase 3)

- [ ] All six services are interface + `service/implementation/<Name>Imp`; only the Imp classes carry `@Service`; nothing outside `implementation/` references an Imp type
- [ ] All services compile with constructor injection (interfaces only), no TamboUI imports
- [ ] All §3.7 tests green
- [ ] Manual check: a scratch `CommandLineRunner` (temporary, then delete) logs
      `detect()` → on this machine it must list at least `winget` (plus scoop/choco if installed)
