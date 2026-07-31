# Migration Plan — dotfile-rs-tui (Rust/Ratatui) → Dotfile-java-TUI (Spring Boot + TamboUI)

> **Audience:** an AI implementer (Claude Haiku) or a junior developer.
> Follow phases **in order**. Never skip a phase. Each phase ends with a
> "Definition of Done" checklist — do not move on until every box passes.

---

## 1. Why this migration

The Rust project `dotfile-rs-tui` (one folder up, at `../`) cannot be built to
a native Windows binary due to driver/security-software issues on this
machine. The Java version's **primary deliverable is a JVM fat jar** running
on **GraalVM JDK 25** (JIT). A GraalVM **native-image** build is a separate,
final phase (11) — if native linking hits the same machine issue, the jar is
still the shipped product.

## 2. What the app does (functional summary of the Rust original)

A cross-platform TUI wrapper around OS package managers:

- Detect OS + installed package managers (winget/scoop/choco/apt/dnf/pacman/
  yay/paru/xbps/brew); show a per-manager command cheat sheet.
- Curated app catalog (`apps.json`) filtered by OS + active manager;
  multi-select → batch install. Live package search and installed-packages
  listing by spawning the manager CLI and parsing its output (incl. winget's
  UTF-16 + `\r`-animated output). Remove flow. Custom package-id entry.
  Streaming install log with interactive stdin (y/n), sudo password entry on
  Linux, and — for interactive managers (pacman, apt, choco…) — suspending
  the TUI to run the command in the real terminal.
- Shell script manager for bash/zsh/fish/nushell/powershell: detect, deploy/
  undeploy a profile script, add/remove the `source` line in the shell
  profile, set/clear a primary shell (`chsh` on Unix), action log.

**The Rust UI was 3 tabs. The Java app does NOT copy that.** It uses a
**lazygit-style layout** (§5). Parity target = *functionality*, not pixels.

## 3. Target stack (fixed decisions — do not re-litigate)

| Concern | Decision |
|---|---|
| JDK | **Java 25 on GraalVM** (managed via **mise**) |
| Language level | Java 25: records, sealed interfaces, pattern-matching `switch`, virtual threads. **No preview features** (no `--enable-preview`) |
| Framework | **Spring Boot 4.1.0**, `web-application-type=none` |
| Async | **Spring's own async stack**: `@EnableAsync` + `@Async` on virtual threads (`spring.threads.virtual.enabled=true`), `CompletableFuture` for one-shot jobs, `ApplicationEventPublisher` for streaming events. No Reactor/WebFlux — KISS: a TUI needs task offloading, not backpressure |
| Build | **Maven** with wrapper; tasks run through **mise** (`mise run …`) |
| TUI library | **TamboUI 0.4.0** (`dev.tamboui`) via the **Toolkit DSL — the fluent API** (`tamboui-toolkit`), **Panama FFM backend** (`tamboui-panama-backend`) — needs `--enable-native-access=ALL-UNNAMED`. The low-level widget/TuiRunner layers are NOT used directly |
| JSON | Jackson as auto-configured by Spring Boot 4.1 (Boot 4 ships Jackson 3 — inject the Boot-managed `ObjectMapper`, adapt imports to what actually autowires) |
| Boilerplate | **Lombok** (Boot-managed version), scoped rules in §4a. **MapStruct: evaluated, NOT used** — models are records mapped straight from JSON; there is no DTO/entity layer to map between. Revisit only if one appears (record the decision change in FEATURE-PARITY.md) |
| Null-safety | **JSpecify** (`org.jspecify:jspecify`) — every package carries a `package-info.java` with `@NullMarked`; fields/params/returns that can genuinely be null are annotated `@Nullable`. Compile-time/IDE tooling only, no runtime behavior |
| Input validation | Popup text inputs (search box, custom package id, sudo password — Phases 7/9) validate via TamboUI's own `dev.tamboui.widgets.form.Validators`/`formField().validate(...)` — already a transitive dependency, no new one needed. **Jakarta/Spring Bean Validation (`spring-boot-starter-validation`): evaluated, NOT added** — `config/AppProperties` currently has one field that already self-heals a blank value in its compact constructor (no fail-fast case to validate) and no consumer yet (Phase 8). Revisit at Phase 8 if a real invalid-config case appears (record the decision change in FEATURE-PARITY.md) |
| Logging | Logback → file only (`debug.log`). Console logging MUST stay off |
| Packaging | Spring Boot fat jar (primary) + GraalVM native-image (Phase 11) |
| Coordinates | `io.github.sampongstarluck:dotfile-java-tui`, base package `com.sampong.dotfile` |

### TamboUI facts (verified July 2026)

Group `dev.tamboui`; latest Central release **0.4.0**; BOM `tamboui-bom`.
Modules used: `tamboui-toolkit` (pulls widgets/core transitively) +
`tamboui-panama-backend`. Docs https://tamboui.dev/docs/main/ (docs track
0.5.0-SNAPSHOT — verify names against the 0.4.0 jars, Rule #0).

**The Toolkit DSL (fluent API) — this is how ALL UI code is written:**
- Apps extend `ToolkitApp` (or drive a `ToolkitRunner`) and override
  `render()` returning an **`Element`** tree; the framework owns the event
  loop, rendering, dispatch, and focus.
- Declarative fluent factories + chained styling:
  `panel("Title", …children).rounded()`, `text("hi").bold().cyan()`,
  `row(…)`, `column(…)`, `columns(…)`, `list(…)`, `table(…)`, `spinner()`
  (built-in animated), `gauge()`, text inputs with state objects.
- **Focus is framework-managed:** `.id("x").focusable()` +
  `.borderColor(…)/.focusedBorderColor(…)`; Tab/Shift-Tab and mouse clicks
  navigate focus out of the box.
- **Keys:** `.onKeyEvent(e -> …)` handlers returning
  `EventResult.HANDLED/UNHANDLED` — attach at the root for globals and on
  panels for feature keys.
- **Overlays/dialogs:** implement `Element` and conditionally render a
  dialog on top inside `render(frame, area, context)` — this is the popup
  mechanism.
- TamboUI's own docs recommend exactly the MVC split this plan uses:
  plain controller classes (query + command methods), views as pure
  functions of state returning `Element`.

> ### ⚠ RULE #0 — verify, don't invent
> In Phase 1 you dump the real class lists from the 0.4.0 jars (`api-*.txt`).
> Any TamboUI name in this plan that doesn't compile → grep those files for
> the real name and use it consistently. Everything that is *not* TamboUI
> (models, services, parsers, state) is exact — type it as written.

> ### ⚠ RULE #1 — no justified workarounds
> If a piece of code needs more than one short comment line to justify a
> workaround, the workaround is wrong: stop and design it properly.
> Corollary: do **not** port the Rust code's commented hacks (e.g. its
> "clear the whole terminal on tab switch" trick or its long `\r`-handling
> apologies). Port the *behavior*; implement it the clean way in TamboUI.
> If the framework can't do something cleanly, record one line in
> FEATURE-PARITY.md and move on — never bury a paragraph-long excuse in code.

## 4. Architecture — Spring MVC, SOLID/KISS/DRY

```
Dotfile-java-TUI/
├── mise.toml                        ← toolchain + tasks (Phase 1)
├── pom.xml  mvnw  .mvn/
├── src/main/resources/
│   ├── application.yml  logback-spring.xml
│   ├── data/apps.json  data/shells.json          ← copied from ../src/json/
│   └── scripts/{bash,zsh,fish,nu,posh}/main_profile.*  ← copied from ../src/scripts/
└── src/main/java/com/sampong/dotfile/
    ├── DotfileTuiApplication.java   ← @SpringBootApplication, hands off to ui.TuiApp
    ├── config/                      ← ALL configuration lives here
    │   ├── AsyncConfig.java         ← @EnableAsync + virtual-thread executor
    │   └── AppProperties.java       ← @ConfigurationProperties(prefix="dotfile"): data dir name, log file…
    ├── base/                        ← tiny abstractions shared across features
    │   ├── PanelView.java           ← interface: void render(Frame f, Rect area, AppState st)
    │   ├── KeyController.java       ← interface: boolean handleKey(KeyEvent k, AppState st)
    │   └── Popup.java               ← sealed interface, permits the concrete popups (§5)
    ├── model/                       ← M of MVC: records + enums, zero logic beyond derivation
    ├── service/                     ← business INTERFACES only (contracts), NO dev.tamboui imports
    │   └── implementation/          ← the @Service classes, named <Interface>Imp
    │       (e.g. service/OsService.java ↔ service/implementation/OsServiceImp.java)
    ├── event/                       ← Spring application events (InstallLogEvent, TaskDoneEvent)
    └── ui/                          ← V + C of MVC; the ONLY package importing dev.tamboui
        ├── TuiApp.java              ← the ONE entry point: extends ToolkitApp, render() = root Element,
        │                              global key handler, focus↔PanelId mapping
        ├── state/                   ← TUI state management (single source of truth, §5a)
        │   ├── AppState.java        ← root: navigation + composed feature states (not a bean)
        │   ├── PlatformState.java  CatalogState.java  SearchState.java
        │   ├── InstalledState.java  InstallState.java  ScriptsState.java
        ├── layout/
        │   └── LazygitLayout.java   ← composes the §5 frame as a fluent row/column Element tree
        ├── component/               ← reusable fluent Element factories (DRY)
        │   ├── Panels.java          ← framed("[n]─Title", focused) lazygit-styled panel()
        │   ├── Lists.java           ← selectable list: rows + cursor + selection markers
        │   ├── Inputs.java          ← single-line input with ▌ cursor (wraps toolkit text input)
        │   ├── Logs.java            ← colored, autoscrolled log column
        │   ├── Popups.java          ← centered overlay wrapper; ConfirmDialog
        │   └── HintBar.java         ← lazygit-style key hints line
        └── feature/                 ← one folder per feature: <X>View (returns Element) + <X>Controller
            ├── status/    managers/    catalog/    search/
            ├── installed/ install/     scripts/    help/
```

Deviation from an earlier draft of this tree: `state/` and `feature/` live under `ui/`
(`ui.state`, `ui.feature.*`), not as top-level siblings — `state/` only exists to back
the `ui/` render loop and no non-UI code touches `AppState`, so nesting it removes a
top-level package that existed for no consumer outside `ui`. `feature/` was already a
factual sibling of `ui/` in the code before this move (a Phase-5 drift from this
diagram); both are now consistent with where the code actually lives.

**Principles applied — the implementer must keep these true:**
- **S**RP: a view class only renders; a controller class only maps keys to
  service calls + state changes; a service never touches the UI.
- **O**CP: `TuiApp` routes to features through the `PanelView` /
  `KeyController` / `Popup` abstractions in `base/` — adding a feature never
  edits routing internals, only the registration list in `TuiApp`.
- **L**SP/ISP: `base/` interfaces stay minimal (one render method, one key
  method) so every implementation is substitutable.
- **D**IP: **every service is an interface** in `service/`; its single
  implementation lives in `service/implementation/` with the `Imp` suffix
  and carries the `@Service` annotation. Controllers/other services inject
  the interface only — no class ever references an `…Imp` type.
  *Exemption:* pure static utility holders (`DecodeUtil`, `TextUtil`,
  `OutputParsers`, `UiText`) are not services and stay final classes of
  static methods.
- **KISS:** no Reactor, no event bus beyond Spring's, no threads beyond the
  Spring async executor, no speculative abstractions.
- **DRY:** every list panel comes from `Lists.selectable`; every popup uses
  `Popups.overlay`; every bordered box is `Panels.framed`; key-hint
  rendering only exists in `HintBar`.

### §4a Lombok rules (scoped — not a free-for-all)

| Annotation | Where | Rule |
|---|---|---|
| `@Slf4j` | any class that logs | **mandatory** — never hand-write `LoggerFactory.getLogger(...)` |
| `@RequiredArgsConstructor` | Spring beans (`…Imp` services, controllers, `TuiApp`, bridges) | **mandatory** — `private final` deps, no hand-written constructors |
| `@Getter` / `@Setter` / `@Builder` / `@Data` / `@Value` | models, state classes | **do NOT use.** Models are records (no Lombok on records); state classes keep public fields + named mutation methods per §5a — blanket setters would break the state discipline |
| `@SneakyThrows` | anywhere | forbidden (hides the error path — Rule #1 territory) |

## 5. UI design — lazygit style (replaces the Rust 3-tab UI)

```
╭─[1]─Status──────────────╮╭─Main─────────────────────────────────────────╮
│ Windows · winget        ││ context view for the focused side panel:     │
╰─────────────────────────╯│  Managers → command cheat-sheet table        │
╭─[2]─Package managers────╮│  Sections → apps of highlighted section      │
│ ● winget                ││  Shells   → shell info + action log          │
│   scoop                 ││ main-view modes: APPS / INSTALLED /          │
╰─────────────────────────╯│              SEARCH_RESULTS / COMMANDS /     │
╭─[3]─Sections────────────╮│              SHELL_INFO                      │
│ Terminal and Shells     ││                                              │
│ Development             ││                                              │
╰─────────────────────────╯│                                              │
╭─[4]─Shells──────────────╮│                                              │
│ ★ ✓ PowerShell 7        ││                                              │
╰─────────────────────────╯╰──────────────────────────────────────────────╯
 1-4 jump · tab cycle · j/k move · space select · d action · / search · ? help
```

- **Left column** (~⅓ width, min 24, max 42 cols — widened from an initial max 34 after
  Phase 7 human review found panel content, e.g. package-manager descriptions,
  routinely clipped at 34): four stacked panels,
  numbered like lazygit. **Right:** one main panel. **Bottom:** 1-line
  context-sensitive key-hint bar.
- **Focus model:** `PanelId { STATUS, MANAGERS, SECTIONS, SHELLS, MAIN }`,
  implemented on top of the **toolkit's own focus system**: each panel is
  `.id("panel-status"|…).focusable()` with
  `.borderColor(darkGray).focusedBorderColor(green)` — Tab/Shift-Tab and
  mouse-click focus come from the framework. `1..4` jump via the toolkit's
  programmatic focus API; `AppState.focused` mirrors the framework focus
  (mapped from the focused element id) so controllers stay UI-agnostic.
  MAIN is entered with `Enter` from SECTIONS, left with `Esc`. Selected row
  bold + reversed.
- **Main view** is an enum `MainView { COMMANDS, APPS, INSTALLED,
  SEARCH_RESULTS, SHELL_INFO }`; derived default follows the focused side
  panel, and `l` (installed) / search results override it.
- **Popups** (sealed `base/Popup` implementations, one modal at a time):
  `SearchInputPopup`, `CustomInputPopup`, `ConfirmActionPopup` (lists the
  exact commands, `y/enter` run, `n/esc` cancel — lazygit-style confirm),
  `SudoPopup`, `InstallLogPopup` (streaming log + stdin input line),
  `HelpPopup` (`?`).
- **This design deletes the Rust PM-picker modal** (the Managers panel *is*
  the picker) and the Rust `searchOrigin` focus bookkeeping (Esc from any
  main view returns to APPS/Sections deterministically). Simpler = correct.
- Keep the Rust glyph vocabulary: `● ▶ ✓ ✗ ★ ◆ ◇ ○ ⚠ ⌕ │ …` + braille spinner.

## 5a. TUI state management (mandatory discipline)

All UI state lives in the `state/` package — **one source of truth**, no
state hidden in views, controllers, or components:

- `AppState` (root) holds only navigation (`focused`, `mainView`, `popup`,
  `running`) and composes one state object per feature:
  `PlatformState`, `CatalogState`, `SearchState`, `InstalledState`,
  `InstallState`, `ScriptsState`. No god-object: a feature touches its own
  state object (plus navigation) and nothing else.
- **Unidirectional flow:** key event → controller **command method** mutates
  state → next `render()` reads state. Views are pure functions of state
  (never mutate); components are stateless; async workers touch only the
  queues/futures inside state objects (§6).
- Toolkit widget state objects (list selection, text-input state, table
  state discovered in Phase 1) are owned by the feature state classes too —
  never created inline in a view.
- Every mutation goes through a named method on the state class or the
  feature controller (e.g. `platform.activateManager(i)`, not field pokes
  from views). Fields are package-private/exposed via methods where
  invariants exist (e.g. cursor clamping lives in the state class — DRY).

## 5b. Startup budget & lazy loading (the 200 ms rule)

**Nothing on the startup path may block > 200 ms.** First frame must appear
immediately after the Spring context is up.

- Phase 1 onward: log a timing line (`log.debug("startup: {} took {}ms", …)`)
  around every startup step. Any step measured **> 200 ms on this machine
  must become lazy**: render a placeholder (`spinner()` + "detecting…") and
  load via a `CompletableFuture` on the async executor, waking the UI with
  `requestRender` when done.
- Applied up front to the known-slow items:
  - **Package-manager detection** (N × PATH probes): lazy — `PlatformState`
    starts `loading=true` with an async detect kicked off at first render;
    Managers panel shows a spinner row until it lands.
  - **Installed-packages list** (spawns the manager CLI): already async +
    auto-loaded once in the background (never on the startup path).
  - **Catalog (`apps.json`) and shell statuses**: lazy on first access
    (measured; JSON parse is expected < 200 ms — verify, don't assume).
- The fallback direction is one-way: fast things may stay eager; slow things
  MUST go lazy. Never "fix" slowness by moving it earlier.

## 6. Concurrency model (Spring-native)

| Need | Mechanism |
|---|---|
| One-shot jobs (search, installed list) | `@Async CompletableFuture<List<SearchResult>>` on `PackageQueryService`; controller keeps the future, polls `isDone()` each frame (KISS — the loop already renders every event) |
| Streaming install/remove log | worker publishes `InstallLogEvent(String line)` via `ApplicationEventPublisher`; an `@Component` listener appends to a `ConcurrentLinkedQueue` owned by `AppState`; UI drains per frame |
| Interactive stdin to child process | `LinkedBlockingQueue<String>` in `AppState`, worker thread forwards to the process |
| Executor | one `AsyncConfig` bean: virtual-thread-per-task (`spring.threads.virtual.enabled=true` + `SimpleAsyncTaskExecutorBuilder.virtualThreads(true)`) |

The UI loop stays single-threaded and never blocks. Workers never mutate
`AppState` fields directly — only the queues/futures above.

Because the Toolkit owns the render loop, background completion must trigger
a re-render: Phase 1 discovers the toolkit's refresh/tick mechanism (its
built-in animated `spinner()` proves one exists) and Phase 5 wires future/
queue draining into it.

## 7. Phase index

| Phase | File | Delivers |
|---|---|---|
| 1 | `plan/phase-01-setup.md` | mise + GraalVM 25 toolchain, Maven project, Spring Boot 4.1 shell, TamboUI/Panama smoke test, file logging, resources, API discovery |
| 2 | `plan/phase-02-models.md` | model records/enums (PanelId, MainView, domain records), JSON loading, tests |
| 3 | `plan/phase-03-platform-services.md` | OS detect, `which`, PM detect, command builders, sudo/interactive classification |
| 4 | `plan/phase-04-parsers.md` | winget UTF-16 decode, ANSI strip, all search/list parsers, fixture tests |
| 5 | `plan/phase-05-state-loop.md` | AppState, base/ interfaces, reusable components, LazygitLayout, TuiApp loop + panel navigation |
| 6 | `plan/phase-06-status-managers.md` | Status panel, Managers panel, Commands main view |
| 7 | `plan/phase-07-catalog-search-installed.md` | Sections panel, Apps/Installed/Search main views, input popups, confirm popup |
| 8 | `plan/phase-08-script-tab.md` | ScriptService + Shells panel + ShellInfo view |
| 9 | `plan/phase-09-async-install.md` | @Async workers, install/remove streaming, sudo, suspend-TUI external run |
| 10 | `plan/phase-10-polish-packaging.md` | Help popup, key-hint bar, fat jar, acceptance run |
| 11 | `plan/phase-11-native-image.md` | GraalVM native-image build (best-effort on this machine) |
| 12 | `plan/phase-12-improvement.md` | **Post-parity improvement, backlog** — MAIN-panel live detail: streaming command log + download progress bar (percentage, current/total size). No Rust source; not scoped yet |

`FEATURE-PARITY.md` maps every Rust behavior to its Java home — tick as you go.

## 8. Working rules for the implementer

1. One phase per session/PR; `mise run test` green before a phase is done.
2. The Rust sources at `../src/**` are the source of truth for *behavior*
   (commands, parsers, paths, state transitions) — **not** for UI structure
   (lazygit layout wins) and **not** for hacks (Rule #1).
3. Pure functions stay pure (static). Services stateless. State lives only
   in `AppState`.
4. No `System.out` after Phase 1 except inside the suspended-TUI external
   runner. Use slf4j → `debug.log`.
5. Windows first: every phase's manual test runs in Windows Terminal
   (PowerShell 7). Unix paths ported but guarded by `OsService` checks.
6. Prefer Java 25 idioms: records, sealed types + exhaustive `switch`
   patterns, `List.of`, text blocks. No preview flags.

## 9. Daily commands (after Phase 1)

```powershell
mise install          # toolchain (GraalVM 25, maven)
mise run dev          # spring-boot:run
mise run test         # unit tests
mise run build        # fat jar
mise run native       # Phase 11 only
```
