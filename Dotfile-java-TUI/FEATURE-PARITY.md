# Feature parity checklist — Rust behavior → Java target

Tick each row when its **behavior** is ported AND verified. The UI is
redesigned to lazygit style (PLAN.md §5), so UI rows map Rust *functionality*
to its new home, not its old look. Paths relative to the Rust root (`../`).

## Toolchain

| Item                                                                                                                                       | Target                                                 | Done |
|--------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------|------|
| mise-managed GraalVM 25 + Maven                                                                                                            | `mise.toml`                                            | ☑   |
| Spring Boot 4.1.0, no web, banner off                                                                                                      | `pom.xml`, `application.yml`                           | ☑   |
| Virtual threads + @EnableAsync                                                                                                             | `config/AsyncConfig`, `spring.threads.virtual.enabled` | ☑   |
| TamboUI 0.4.0 **Toolkit DSL (fluent API)** + Panama FFM backend (Windows verified)                                                         | `tamboui-toolkit` in pom + Phase 1 smoke test          | ☑   |
| All UI written as fluent `Element` trees (no immediate-mode calls outside `Popups.overlay`)                                                | `ui/**`                                                | ◐    |
| Every service = interface in `service/` + `<Name>Imp` in `service/implementation/` (only Imp has `@Service`)                               | all phases                                             | ◐    |
| State management: composed `state/` classes, unidirectional flow, no state in views/components                                             | `state/*`                                              | ◐    |
| 200 ms rule: no synchronous startup step > 200 ms; slow loads lazy with spinner (PM detection async)                                       | Phase 5/6/10 audit                                     | ◐    |
| Lombok scoped rules: `@Slf4j` + `@RequiredArgsConstructor` mandatory on beans; no `@Data`/`@Setter`/`@SneakyThrows`; none on records/state | PLAN.md §4a audit                                      | ☑   |
| MapStruct: evaluated, not used (no DTO layer) — decision recorded; revisit only if a mapping layer appears                                 | —                                                      | ☑   |
| `--enable-native-access=ALL-UNNAMED` everywhere (mise env, boot plugin, launcher)                                                          | Phase 1/10                                             | ☑   |
| JSpecify null-safety: `@NullMarked` on every package, `@Nullable` on genuinely-nullable fields/params/returns                              | `pom.xml` + `package-info.java` per package            | ☑   |
| Jakarta/Spring Bean Validation: evaluated, not added (no fail-fast case exists yet) — decision recorded; revisit at Phase 8               | PLAN.md §3                                             | ☑   |

## Models & config (Phase 2)

| Rust source | Java target | Done |
|---|---|---|
| `src/models/os.rs` (types) | `model/OperatingSystem`, `model/LinuxDistro` | ☑ |
| `src/models/package_manager.rs` (binary/label/commands data) | `model/PackageManager`, `model/PmCommand` | ☑ |
| `src/models/apps.rs` | `model/AppSection`, `model/AppEntry` | ☑ |
| `src/models/shell.rs` + `shell_model.rs` | `model/ShellEntry`, `model/ShellsFile` | ☑ |
| `SearchResult`, `ShellStatus`, `DotfileConfig` (snake_case JSON) | `model/*` | ☑ |
| Rust `TabModel`/`AppFocus` | **replaced** by `PanelId` + `MainView` (sealed `Popup` deferred to Phase 5) | ☑ |
| — | `config/AppProperties` (data dir name, default `dotfile-rs`) | ☑ |

## Services (Phases 3, 4, 8)

| Rust source | Java target | Done |
|---|---|---|
| `os.rs::detect` + `/etc/os-release` | `service/OsService` | ☑ |
| `which` crate | `service/PathService.which` (PATHEXT-aware) | ☑ |
| `package_manager.rs::detect/candidates_for` | `service/PackageManagerService` | ☑ |
| `src/service/system_service.rs` | `service/SystemService` | ☑ |
| `src/service/install_service.rs` | `service/InstallCommandService` | ☑ |
| `src/service/app_service.rs` | `service/AppCatalogService` | ☑ (Phase 2; `readShellsJson` also covers `script_service.rs::read_shells`) |
| `search_service.rs` command tables + hint | `service/SearchService` | ☑ (command tables only; parsers are Phase 4) |
| `src/utils/decode_util.rs` | `service/DecodeUtil` | ☑ |
| `src/utils/text_util.rs` | `service/TextUtil` | ☑ |
| `search_service.rs` 9 search + 8 list parsers | `service/OutputParsers` | ☑ |
| `app_tab.rs::build_commands/build_remove_commands/selected_display_names` | `service/CommandPlanner` | ☑ |
| `src/service/script_service.rs` (all fs/profile/config logic) | `service/ScriptService` | ☑ |
| `src/logging.rs` (file-only logging) | `logback-spring.xml` → `debug.log` | ☑ |

## UI shell (Phase 5)

| Rust behavior | Java target (lazygit design) | Done |
|---|---|---|
| `src/app.rs::App` state + reset logic | `state/AppState` (incl. `activateManager` reset) | ☑ |
| `src/main.rs` loop: press-only keys, help gating, quit, resize | `ui/TuiApp` | ☑ |
| Tab switching | toolkit focus system: `.id().focusable()`, Tab/Shift-Tab/click native, `1-4` programmatic; Enter/Esc into MAIN deferred to Phase 7 (SECTIONS controller) | ☑ |
| loading spinner | toolkit built-in animated `spinner()` (Managers panel while `platform.detecting`) | ☑ |
| `src/ui/layout/layout.rs` | `ui/layout/LazygitLayout` (fluent row/column composition) | ☑ |
| shared panel/list/input/log/popup drawing | `ui/component/*` factories (Panels, Lists, Inputs, Logs, Popups, HintBar, UiText) | ☑ |
| `base/` contracts (FeatureView, KeyController, sealed Popup) | `base/*` | ☑ |
| background completion → re-render | `PlatformQueryService` `@Async` future + `runner().runOnRenderThread(...)` + `requestRender()`; default 40ms tick also covers redraw | ☑ |

## Features (Phases 6–9)

| Rust behavior | Java target | Done |
|---|---|---|
| home sidebar (PM list) + platform info | `feature/status/StatusPanel` + `feature/managers/ManagersPanel` | ☑ |
| home command cheat-sheet table + scroll | `feature/managers/CommandsView` | ☑ |
| PM picker modal | **deleted** — Managers panel is the picker | ☑ |
| catalog sections + apps list, `[✓]`/`[I]`, badge | `feature/catalog/` SectionsView + AppsView | ☑ |
| custom package input | `CustomInputPopup` | ☑ |
| search input + results (3-col, loading, empty) | `feature/search/` SearchInputPopup + SearchResultsView | ☑ |
| installed view (red remove styling, refresh) | `feature/installed/InstalledView` | ☑ |
| install/remove decision tree | `feature/install/InstallController` | ☑ |
| (new) explicit confirm before running commands | `ConfirmActionPopup` (lazygit-style) | ☑ |
| sudo password modal | `SudoPopup` (masked input added) | ☑ |
| install modal: colored log, autoscroll, stdin input | `InstallLogPopup` + `Logs.colored` | ☑ |
| scripts tab (shells list, info, log, all keys) | `feature/scripts/` ShellsView + ShellInfoView + ScriptsController | ☑ |

## Async & external (Phase 9 — Spring-native)

| Rust behavior | Java target | Done |
|---|---|---|
| `run_search` thread + mpsc | `PackageQueryService.search` `@Async` → `CompletableFuture`, polled per frame | ☑ |
| `run_list_installed` | `PackageQueryService.listInstalled` | ☑ |
| streamed install/remove + stdin relay + sudo -S | `InstallExecutionService.runStreaming` + `InstallLogEvent` + `InstallLogBridge` | ☑ |
| `drain_stdout_to_log` (\r\n split, ansi strip, noise filter) | `InstallExecutionServiceImp.drainStdout` | ☑ |
| `main.rs::run_in_terminal` suspend/inheritIO/restore/reset | `ui/ExternalRunner` | ☑ |

## Chrome & packaging (Phases 10–11)

| Item | Target | Done |
|---|---|---|
| status-bar hints + help overlay (single source) | `ui/Bindings` → `HintBar` + `HelpPopup` | ☑ |
| mouse (clicks, wheel) | click-to-focus works (toolkit-native); row-click/wheel-scroll extras skipped, see deviations | ◐ |
| fat jar + launcher + README | Phase 10 (`README.md`, `dotfile.cmd`) | ☑ |
| GraalVM native-image | Phase 11 — builds; interactive run needs human confirmation (see below) | ◐ |

## Resources & interop

| Item | Target | Done |
|---|---|---|
| `src/json/apps.json`, `src/json/shells.json` | `resources/data/` | ☑ |
| `src/scripts/**` (5 profiles) | `resources/scripts/**` | ☑ |
| `%APPDATA%\dotfile-rs` + `config.json` snake_case kept Rust-compatible | `ScriptService` + `DotfileConfig` | ☑ |

## Deliberate deviations (append as discovered)

- UI restructured to lazygit panels; Rust tab/focus bookkeeping
  (`TabModel`, `AppFocus`, `searchOrigin`, PM-picker modal) deleted by design.
- Install/remove now asks an explicit confirm popup before executing (the
  Rust ran immediately on `d`).
- `winget list` decoded with the UTF-16-aware decoder (Rust used lossy UTF-8)
  — strict improvement.
- Primary deliverable is the JVM fat jar on GraalVM 25; native-image is
  best-effort (Phase 11).
- `PathService.which` on Windows checks existence with `LinkOption.NOFOLLOW_LINKS`
  instead of `Files.isRegularFile` — `winget.exe` under `WindowsApps` is an App
  Execution Alias reparse point NIO cannot stat through when following links,
  which made `Files.isRegularFile` report it as absent.
- `parse_apt_search` (`../src/service/search_service.rs:256`, ported to
  `OutputParsers.parseAptSearch`) has a pre-existing quirk: it splits the line
  on the first `/` and takes the FIRST whitespace token of the remainder as
  the version. For real `apt search` output (`pkg/suite version arch`) that
  first token is the suite codename (e.g. `noble`), not the version — the
  actual version is the *second* token of the remainder. `parse_apt_list`
  does not have this bug (it indexes the whitespace tokens of the whole line
  correctly). Ported byte-perfect per Rule #1/PLAN.md §8.2 rather than
  "fixed" — verified by simulating the Rust `splitn(2, '/')` + `split_whitespace()`
  semantics; the phase-04-parsers.md doc's own fixture answer for this case
  does not match the real Rust source's output.
- **Phase 5:** the sealed `Popup` interface's `permits` subtypes
  (`HelpPopup`, `SearchInputPopup`, `CustomInputPopup`, `ConfirmActionPopup`,
  `SudoPopup`, `InstallLogPopup`) live directly in `base/`, not in their
  feature packages as PLAN.md §5.1 originally sketched — Java requires a
  sealed type's permitted subtypes to share its package when the project has
  no `module-info.java` (unnamed module), and adding JPMS to a Spring Boot
  fat jar (interacting with Phase 11's native-image) was judged out of scope
  for this phase. Each record documents this in its Javadoc.
- **Phase 5 (superseded — see Phase 6 note below):** `LazygitLayout`'s side
  column originally used a fixed `.min(24)` floor with no upper bound.
  Human verification of Phase 6 (screenshot on a wide terminal) showed this
  wasn't a minor overflow as assumed: Cassowary's `Min` constraint competes
  for leftover space the same way `Fill` does, so the side column and the
  main panel split roughly 50/50 instead of the side column staying near a
  third. Fixed in Phase 6 — see below.
- **Phase 6:** fixed the Phase 5 side-column-width issue above: `LazygitLayout`
  now wraps `body` in `ui/component/Responsive` and computes an explicit
  `.length(clamp(area.width() / 3, 24, 34))` from the real body area at
  render time, giving the true `min(24)/max(34)` clamp PLAN.md §5 specifies.
  `Responsive` also now delegates `id()`/`isFocusable()`/key & mouse event
  handlers/`renderedArea()` to its last-built child (matching the toolkit's
  own `LazyElement`), since it now sits at the frame root rather than only
  inside leaf panels.
- **Phase 5:** `PackageManagerService.detect()` (blocking PATH probes) is
  wrapped by a new `service/PlatformQueryService` (`@Async` `CompletableFuture`)
  — not itself in PLAN.md's Phase 3 file, but required by §5.6's
  `pmQuery.detectManagers()` lazy-load and consistent with the Phase 3
  service/interface + `Imp` pattern.
- **Phase 5 verification:** `mvn compile`/`test` pass and Spring context
  wiring succeeds end-to-end (all beans, incl. `TuiApp`, resolve cleanly).
  The full-screen Panama backend itself could not be interactively driven
  from the agent session — there is no `tmux` on this Windows machine, and a
  background-launched process here has no real console handle
  (`BackendException: Failed to get input console mode`, the same failure
  any TUI would hit launched this way). This matches Phase 1's own
  precedent: the Definition of Done's focus/spinner/resize/popup/quit items
  needed a human to run `mise run dev` in an actual Windows Terminal window —
  since confirmed by a human run; all rows verified.
- **Phase 6:** width-dependent degrade rules (Managers panel `binary()` vs
  `label()` + description at ≥30 cols; Commands table's 45%/15-col split and
  the `w<20 || h<4` "terminal too small" guard) need the real rendered
  `Rect`, which only exists after the Cassowary layout solver runs — after
  the fluent tree is already built. Added `ui/component/Responsive`
  (`Element` implemented directly, the same technique PLAN.md already
  sanctions for popups) to defer that content decision to its own
  `render(Frame, Rect, RenderContext)` call, where the real area is known.
- **Phase 6:** the Commands table's `"[<scroll+1>/<total>]"` indicator is
  rendered as a content line above the table, not baked into the MAIN
  panel's border title — `Panel.title()` takes a fixed `Line` set before
  layout runs, so it can't depend on `visibleRows` (which needs the render-
  time area, same root cause as the `Responsive` note above). The title
  itself (`"Commands — <label>"`) *is* wired dynamically per main view via
  a new `FeatureView.title(AppState)` default method, since that part only
  depends on state, not area.
- **Phase 6 verification:** `mvn compile`/`test` pass. `mise run dev` was
  launched from the agent session and reached `ToolkitApp.run()` (all beans,
  incl. the new `Responsive`-based views, resolve cleanly), failing only at
  the same `BackendException: Failed to get input console mode` recorded in
  Phase 5 — not a regression. The interactive DoD rows (spinner timing,
  cursor/`●` movement, table scroll/clamp, hint-bar focus updates) were
  subsequently confirmed by a human running `mise run dev` in an actual
  Windows Terminal window; all rows verified.
- **Phase 7:** added `AppState.pendingFocus` + `AppState.requestFocus(PanelId)` — controllers
  (e.g. `SectionsController` Enter-into-MAIN/APPS, `AppsController`/`InstalledController` Esc-back)
  cannot touch the toolkit's own focus manager directly (it lives on `TuiApp`/`ToolkitRunner`,
  not on `AppState`), so a controller requests a jump by setting this field and `TuiApp.frameTick()`
  applies it via `focusPanel()` *before* `syncFocus()` reads the toolkit's focus back — otherwise
  the very next `syncFocus()` call would immediately overwrite the request with the toolkit's
  still-unchanged actual focus.
- **Phase 7:** introduced `service/PackageQueryService` + `PackageQueryServiceImp` (stub body) to
  wire the `st.search.future`/`st.installed.future` polling + drain plumbing (PLAN.md §6) ahead of
  Phase 9's real process spawning, per this phase's own instruction ("Async execution stays
  stubbed until Phase 9, but all wiring goes in now"). `search()` returns one fake row so the
  search flow is visually verifiable this phase; `listInstalled()` returns an empty list.
- **Phase 7:** `ConfirmActionPopup`/`Popups.confirm` gained a `warning` line parameter (the Phase 5
  stub signature didn't have one) to render `"⚠ runs with <mgr>"` per §7.6. `SearchInputPopup`/
  `CustomInputPopup` switched from the Phase 5 placeholder `StringBuilder` field to the toolkit's
  real `TextInputState` (+ `Toolkit.handleTextInputKey`), matching `ui/component/Inputs.line`'s
  actual signature from Phase 5. `SearchInputPopup` now carries an `onSearch` callback (mirroring
  `ConfirmActionPopup`'s existing `onConfirm` `Runnable`) rather than a query-string field, so the
  controller that opened it supplies the `PackageQueryService` call as a closure.
- **Phase 7:** `ui/feature/catalog/CatalogActions` is the single shared home for install/remove
  confirm-popup construction, search/custom-popup construction, and the installed-list refresh
  trigger — reused by `SectionsController`, `AppsController`, `SearchController`, and
  `InstalledController` (all four mutate the one shared `st.catalog.selectedIds`, matching the
  Rust source's single `app_selected_ids`), per this phase's own "pick one place, DRY" instruction.
- **Phase 7 verification:** `mvn compile`/`test` pass (44 tests green, incl. 5 new
  `CommandPlannerTest` cases). `mise run dev` reached `ToolkitApp.run()` (all new beans resolve,
  `TuiApp.onStart()` runs) and failed only at the same `BackendException: Failed to get input
  console mode` recorded in Phases 5/6 — not a regression. The interactive DoD rows need a human
  to run `mise run dev` in an actual Windows Terminal window.
- **Phase 7 (post-review):** `LazygitLayout`'s side column max widened from `34` to `42` cols
  (`min(24)/max(42)`) after a human running `mise run dev` reported the Managers panel routinely
  clipping package-manager descriptions (e.g. "Windows built-in, lar…"). Widening alone still
  wasn't enough (the description competes with the marker + label on one line), so
  `ManagersView` now gives each manager's description its own indented second line instead of
  packing it onto the label line — `UiText.truncate` (ellipsis) remains as a safety net for
  terminals too narrow to fit even a full line. `ManagersView.panelHeight(pmCount)` (2 lines per
  visible row + border) replaces the old 1-line-per-row height calc in `LazygitLayout`. PLAN.md
  §5's column-width spec updated to match.
- **Phase 7 (human verification, post-review):** the first live `mise run dev` run in a real
  Windows Terminal surfaced three pre-existing bugs, none introduced by Phase 7's own diff:
  1. **Panel-collapse layout bug.** `LazygitLayout` sized each panel with `column(panel).fill()`/
     `.length(n)` — a throwaway wrapping `Column` used only to attach a constraint. That wrapper
     reports the constraint correctly to *its own* parent, but then independently re-derives a
     constraint for its single child via `Column.renderContent`'s `child.constraint() == null`
     fallback, which queries `child.preferredSize()`. A bordered `Panel` always reports a real,
     content-sized `preferredSize()` (never "unknown"), so the panel got pinned to
     `Constraint.length(contentHeight)` instead of stretching — worst on `[3]-Sections` and `MAIN`
     (both `.fill()`), which rendered as a small bordered box with a large blank, borderless gap
     filling the rest of their allocated slot, even on a maximized terminal. Fixed by adding
     `ui/component/Sized` (constraint exposed directly + a pass-through `render` that hands the
     whole given area straight to the child, skipping the inner re-derivation) and using it for
     every panel slot in `LazygitLayout`.
  2. **Catalog/installed startup race.** `TuiApp.catalogTick()`'s one-time `apps.json` load (and
     the installed-list auto-load) ran on the very first frame, before the async
     `PlatformQueryService.detectManagers()` future resolved — filtering against
     `activeBinary()`'s `"unknown"` placeholder and permanently locking `st.catalog.apps` empty
     (`[3]-Sections` showed "no apps for winget" forever; nothing re-triggers the load afterward,
     unlike an explicit manager switch, which goes through `AppState#activateManager`'s
     `catalog.reset()`). Fixed by gating both loads on `!st.platform.detecting`.
  3. **Popup width bug (framework).** `dev.tamboui.toolkit.elements.DialogElement` has two
     disagreeing width calculations: `preferredSize()` correctly measures the widest child, but
     nothing reads it — the dialog's `constraint()` unconditionally returns `Constraint.fill()`,
     and the width actually used at render time (`calculateWidth`) ignores children completely
     (title length vs. a 20-col floor only). The search popup's hint line clipped mid-word as a
     result. Can't patch the library; worked around in `ui/component/Popups` by pinning each
     dialog's `fixedWidth` from its own (correct, otherwise-dead) `preferredSize()` call —
     applies uniformly to every popup (search, custom-id, sudo, help, confirm, install log).

  All three fixed, human re-verified every Phase 7 DoD row afterward (`plan/phase-07-catalog-search-installed.md`).
- **Phase 8:** `model/ShellStatus` widened beyond the Rust struct's 4 fields (`entry`, `detected`,
  `deployed`, `targetPath`) with `binary`/`profilePath`/`sourceHint`. The Rust `render_info` calls
  `shell_binary`/`shell_profile_path`/`source_hint` directly since Rust has no view/service-call
  boundary; our `ui/` package does (PLAN.md §4 SRP — views are zero-service-call, a Phase 8 DoD
  row). These three values are pure, id-derived, and side-effect-free, so `ScriptServiceImp
  .loadShellStatuses()` computes them once per shell and `ShellInfoView` just reads the record —
  `ScriptService` stays the single owner of all path-computation logic.
- **Phase 8:** the lazy `st.scripts.shells` load lives in a new `TuiApp.scriptsTick()` (mirroring
  the existing `catalogTick()`), not inline in `ShellsView.render()` like the Rust `script_render`
  does — views must stay pure/non-mutating (PLAN.md §5a), and `catalogTick()` already established
  this "lazy load in a `TuiApp` tick method" pattern in Phase 7. Fully synchronous (local filesystem
  reads only), so unlike `catalogTick()`'s async-future wait, there's nothing to poll.
- **Phase 8:** `ScriptsState` gained `explicitPrimaryShell` (the raw `config.json` value) alongside
  the existing `primaryShell` (effective: explicit-or-detected-default) — needed so `ShellInfoView`
  can render the Rust `render_info`'s three-way primary-line distinction ("★ set as primary" /
  "◇ system default" / "—") from state alone, without calling `ScriptService.readConfig()` itself.
- **Phase 8 verification:** `mvn compile`/`test` are green (50 tests, incl. 6 new
  `ScriptServiceImpTest` cases, all forcing the Unix/XDG path branch against `@TempDir` — the
  Windows branch reads the real `%APPDATA%` env var directly, which a unit test must never touch).
  `mise exec -- mvn -q spring-boot:run` reached the same `ToolkitApp.run()` → backend-creation point
  as every prior phase (all new beans resolve, `TuiApp.onStart()` runs), failing only at the same
  `BackendException: Failed to get input console mode` recorded since Phase 5 — not a regression.
  A human has since run `mise run dev` in a real Windows Terminal window and confirmed every
  interactive DoD row in `plan/phase-08-script-tab.md` (Shells panel listing, deploy, idempotent
  re-deploy, undeploy, set/clear primary shell). Phase 8 is fully done.
- **Phase 9:** `ToolkitRunner`/`TuiRunner` (verified against the 0.4.0 sources jar) expose no
  pause/resume API — only `create()`/`close()`. Per the phase-09 plan's own note, `TuiApp` now
  overrides `ToolkitApp.run()` (not inherited) with a loop: each `super.run()` call owns one
  `ToolkitRunner` lifecycle end-to-end; `frameTick()` calls `quit()` when `st.install.runExternal`
  is non-empty (`externalRunPending` guards against calling it twice), and once `super.run()`
  returns — after `TuiRunner.close()`'s `cleanup()` has already left the alt screen and disabled
  raw mode — `run()` hands off to the new `ui/ExternalRunner` (port of `main.rs::run_in_terminal`,
  the only class besides the framework allowed `System.out`/`System.in`) and then loops back into
  `super.run()`, opening a fresh runner. `onStart()` re-runs on each loop iteration (idempotent:
  `platformDetectStarted` still guards the one-shot async platform detect; panel/main controller
  maps are just overwritten with fresh, stateless instances) rather than adding a second
  init-vs-reinit code path.
- **Phase 9:** `InstallLogBridge` (the `@EventListener` that appends `InstallLogEvent`s to the
  queue `TuiApp.frameTick()` drains, then wakes the render loop) takes its `Runnable` render-waker
  via a setter (`TuiApp.onStart()` calls `installLogBridge.setRenderWaker(this::requestRender)`)
  instead of constructor injection. Constructor-injecting `TuiApp` into `InstallLogBridge` would
  create a circular Spring bean dependency (`TuiApp` already needs `InstallLogBridge` to build
  `InstallController`); the setter breaks the cycle without weakening the "only one listener
  appends" SRP the phase-09 plan calls for.
- **Phase 9:** the Rust worker's cancellation-on-close comes for free from `mpsc` channel
  semantics (`Esc` implicitly drops the receiver, so the detached worker's next `tx.send()` fails
  and `drain_stdout_to_log` returns early). `InstallLogBridge.sink` is a `ConcurrentLinkedQueue`
  with no such "closed" signal — appends always succeed — and it is a single field shared across
  every install/remove run, so a late event from a worker abandoned via `Esc` would otherwise leak
  into whatever queue the *next* install attaches. Added `InstallLogBridge.detach()`, called from
  `InstallController.onClose()` (the same place `InstallLogPopup`'s Esc handler already routes
  through), so abandoned-worker events are dropped instead of cross-contaminating a later run. The
  worker itself still runs its remaining commands to completion detached, matching the Rust.
- **Phase 9:** `SudoPopup`'s password field was speced as a raw `StringBuilder` (matching the
  Phase 7 stub) but is implemented with the toolkit's real `TextInputState` instead — masking is
  just `"•".repeat(password.length())` at render time, and it reuses `Toolkit.handleTextInputKey`
  rather than hand-rolling char/backspace handling a second time (`CustomInputPopup`/
  `SearchInputPopup` already established this pattern in Phase 7). Pure DRY/consistency
  improvement, no behavior change.
- **Phase 9 verification:** `mvn compile`/`test` are green (50 tests, unchanged — phase-09's DoD
  is entirely interactive/process-spawning behavior with no new fixture-style unit tests called
  for). `mise exec -- mvn -q spring-boot:run` reached the same point as every prior phase — all
  new beans (`SystemService` was already Phase-3; `InstallExecutionService`/
  `InstallExecutionServiceImp`, `InstallLogBridge`, and `TuiApp`'s new constructor deps resolve
  cleanly, `TuiApp.onStart()` builds `InstallController` and wires it into every catalog-adjacent
  controller) and failed only at the same `BackendException: Failed to get input console mode`
  recorded since Phase 5 — not a regression. The Definition of Done in
  `plan/phase-09-async-install.md` is entirely live process-spawning behavior (real `winget
  search`, real install/remove streaming, sudo, suspend/resume with choco or a temporarily-forced
  interactive manager) that needs a human to run `mise run dev` in a real Windows Terminal window
  with real package managers installed — none of it can be driven from this agent session.
- **Phase 9 (human verification, post-review):** a human ran `mise run dev` in a real Windows
  Terminal window with real package managers installed and confirmed every interactive DoD row in
  `plan/phase-09-async-install.md` (real `winget search`, real installed-list `[I]` markers,
  install streaming to `═══ All done ═══`, remove flow, Esc-mid-install, external-command
  suspend/resume round-trip). Phase 9 is fully done.
- **Phase 10:** `ui/Bindings` is the single canonical source for every keybinding hint (PLAN.md
  §10.1) — `HintBar` (bottom bar) and `HelpPopup` both render from it, replacing the old ad-hoc
  `TuiApp.keyHints()` switch (which never accounted for popup states, so the bottom hint bar stayed
  on the underlying panel's hints even while a popup was open) and the Phase-5 stub `HelpPopup`'s
  hard-coded 2-line text. `HintBar`'s own `Binding` record was removed in favor of
  `Bindings.Binding` to avoid two near-duplicate types (DRY). The generic inline hint lines that
  `SearchInputPopup`/`CustomInputPopup`/`SudoPopup`/`InstallLogPopup` had accreted in their own
  `view()` bodies (Phases 7/9) were removed for the same reason — §10.1's "nothing else may print
  hints" — since the bottom `HintBar` now always reflects the open popup's context.
  `ConfirmActionPopup`/`Popups.confirm`'s `"[y] Run   [n] Cancel"` row was kept: it renders
  caller-supplied labels (not fixed generic text), so it is dialog content, not a duplicate hint.
- **Phase 10:** mouse row-click-to-move-cursor and wheel-scroll-on-hover (PLAN.md §10.3's optional
  extras) were not implemented. `dev.tamboui.tui.event.MouseEvent` does expose the needed primitives
  (`scrollUp`/`scrollDown`, `x()`/`y()`, `isClick()`), but `ui/component/Lists.selectable` renders
  each list as a fresh, non-persistent `Column` snapshot every frame (via `Responsive`, Phase 9
  deviation) rather than a stateful widget tree — there is no existing per-row `renderedArea()` to
  hit-test a click or scroll against without a real redesign of that rendering model, which is out
  of scope for a polish phase (Rule #1: no piecemeal hack). Click-to-focus on panels (verified
  working since Phase 5, toolkit-native) is unaffected. Stray mouse events elsewhere are harmless:
  no code in this app attaches an `onMouseEvent` handler outside `Sized`/`Responsive`'s pass-through
  delegation, so an unhandled `MouseEvent` just falls through the framework's default (no crash) —
  confirmed by reading `Element.handleMouseEvent`'s default and the fact `TuiApp` only wires
  `onKeyEvent` at the root.
- **Phase 10 (post-review, human found Help unscrollable):** the first cut of `HelpPopup` (see the
  note above) rendered its full ~60-line generated body with no scrolling, so on any terminal
  shorter than that it clipped at the bottom with no way to see the rest — found by human review
  right after the initial Phase 10 pass. Fixed by giving `HelpPopup` a real
  `dev.tamboui.widgets.scrollbar.ScrollbarState` record component (the same "toolkit widget-state
  object owned by the popup" pattern `TextInputState` already established on the input popups) and
  windowing its body through `ui/component/Responsive`, exactly like `Lists.selectable`'s own
  cursor-following window (Phase 9 deviation) — `j`/`k`/arrows scroll one line, `pgup`/`pgdn` one
  page (`Bindings.CTX_HELP_POPUP` updated to advertise this). Two `DialogElement` internals drove
  the implementation, confirmed by reading its decompiled/sources-jar code directly rather than
  guessing:
  1. `DialogElement.calculateHeight()` (the one actually used at render time) sums children's real
     `preferredSize()` when no `.length(n)` is set — unlike the already-known width bug, height
     alone isn't broken. But `Responsive.preferredSize()` deliberately returns `Size.UNKNOWN`
     (its content depends on the render-time area), so a Responsive body would size the dialog to
     just 1 row. Fixed by pinning an explicit `.length(30)`; `renderContent`'s own
     `Math.min(dialogHeight, area.height())` still clamps that safely down on a short terminal, so
     30 is a "use a generous chunk of the screen" upper bound, not a hard requirement.
  2. `DialogElement.calculateWidth()` (also render-time-real, confirmed alongside the height read)
     ignores children entirely regardless of `.length()`/scrolling — the pre-existing Phase 7 bug.
     `Popups.overlay`'s own `preferredSize()`-based width fix can't see into a `Responsive` child
     for the same `Size.UNKNOWN` reason as above, so `HelpPopup` measures width from its full,
     unwindowed line list via a new `Popups.measureWidth(title, content...)` (factors out the same
     `dialog(...).preferredSize(-1,-1,RenderContext.empty()).width()` call `overlay`'s `sized()`
     already made, so both paths share one width-measuring formula) and overrides the real
     (Responsive-bodied) dialog's `.width(...)` with that measured value after the fact.
- **Phase 11:** `mise run native` (`mvn -Pnative native:compile`) succeeds — `target/dotfile-java-tui.exe`,
  67.7MB, ~3 min build. Two real native-image issues were hit and fixed (not just the "same
  `BackendException` as every prior phase" precedent — these are new, native-image-specific):
  1. `tamboui-tui-0.4.0.jar` ships its built-in key-binding sets as plain classpath resources
     (`dev/tamboui/tui/bindings/{standard,vim,emacs,intellij,vscode}.properties`) with **no**
     native-image metadata of its own — unlike `tamboui-panama-backend`, which ships a
     `reachability-metadata.json` for its FFM downcalls (verified by inspecting both jars directly:
     `tamboui-panama-backend` has a `META-INF/native-image/.../reachability-metadata.json`,
     `tamboui-tui` has no `META-INF/native-image` directory at all). Without a hint, the first native
     build failed at `BindingSets.<clinit>` → `RuntimeIOException: Built-in bindings not found:
     standard.properties`. Fixed with a `RuntimeHintsRegistrar` (`NativeHints.TamboUiResources`,
     wired via `@ImportRuntimeHints`) registering `dev/tamboui/tui/bindings/*.properties` — the
     standard Spring AOT mechanism for classpath-resource hints, not a native-image config file
     bolted on separately.
  2. `WindowsTerminal`'s FFM downcalls close a **shared** `Arena` (`Arena.ofShared`), which
     native-image disables by default: `UnsupportedFeatureError: Support for Arena.ofShared is not
     active: enable with -H:+SharedArenaSupport`. Fixed by adding that buildArg to the `native`
     Maven profile. This **replaces** PLAN.md §11.2's speculative `-H:+ForeignAPISupport` guess —
     that flag isn't what's needed; the FFM downcalls themselves are already covered by
     `tamboui-panama-backend`'s own shipped `reachability-metadata.json` (21 downcalls + 1 upcall
     registered automatically per the build log), it's specifically the shared-arena-close path that
     needs unlocking. GraalVM currently flags `-H:+SharedArenaSupport` as experimental and warns a
     future release will require pairing it with `-H:+UnlockExperimentalVMOptions` — added that pairing
     proactively in `pom.xml` rather than waiting for it to become a hard requirement on a GraalVM
     upgrade.
  3. `config/NativeHints.java` also carries `@RegisterReflectionForBinding` for the six
     Jackson-mapped model records PLAN.md §11.3 names (`AppEntry`, `AppSection`, `ShellEntry`,
     `ShellsFile`, `DotfileConfig`, `SearchResult`) — added proactively per the plan; no reflection
     failure was actually observed for these (Spring AOT's own processing of Jackson-bound types may
     already have covered some), but registering them explicitly is cheap and matches the plan's
     instruction.
  After these two fixes, launching `target\dotfile-java-tui.exe` from this agent session reaches
  `dev.tamboui.terminal.BackendFactory.create()` and fails with the same class of error every prior
  phase has hit here — `BackendException: All backend providers failed to create a backend... Failed
  to get output console mode` — i.e. no regression, no new blocker: the native binary gets exactly as
  far as the JVM jar does when launched without a real console handle. **This is genuine parity with
  the fat jar's own startup path, not full verification** — the interactive acceptance walkthrough
  (PLAN.md §10.5, re-run against the native exe per phase-11's own Definition of Done) still needs a
  human to run `target\dotfile-java-tui.exe` in a real Windows Terminal window before Phase 11 can be
  marked fully done.
- **Phase 11 (post-review, warnings follow-up):** three items from the native-image build log's own
  warnings/recommendations were investigated and addressed:
  1. `Warning: Option 'DynamicProxyConfigurationResources' is deprecated...` traced to
     `micrometer-core-1.17.0.jar`'s `META-INF/native-image/io.micrometer/micrometer-core/proxy-config.json`
     (the legacy pre-unified-metadata native-image config format), pulled in transitively via
     `spring-boot-starter-actuator`. Grepping the whole source tree found **zero** uses of
     actuator/micrometer anywhere in the codebase, no `management.*`/`actuator.*` config in
     `application.yml`, and the app is `web-application-type: none` besides — the dependency was dead
     weight from the start, not in PLAN.md §3's dependency table either. Removed it from `pom.xml`
     entirely rather than working around the warning; that's the actual root cause, an upstream
     library's legacy metadata format is not something this project can or should patch around.
  2. `-H:+SharedArenaSupport` experimental-flag warning: addressed as noted above (paired with
     `-H:+UnlockExperimentalVMOptions` proactively).
  3. **PGO** (the build log's own "PGO: Use Profile-Guided Optimizations (`--pgo`) for improved
     throughput" recommendation) was implemented as a full two-step workflow (`mise run
     native-instrument` → exercise the exe → `mise run native-pgo`), parametrizing the single `native`
     Maven profile via `-D`-overridable properties (`native.imageName`, `native.pgoArg`) rather than
     separate profile ids — a first attempt used separate `native-instrument`/`native-pgo` profile ids
     and both failed with "Please specify class... containing the main entry point", because
     `spring-boot-starter-parent` ships its own built-in profile literally named `native` that wires
     `spring-boot:process-aot` + auto-detected `mainClass` for `native-maven-plugin`, and Maven only
     merges that wiring into a child profile of the **same id**. This is also why the original Phase 11
     baseline profile never needed an explicit `mainClass` — it was silently inheriting the parent's.
     Second, real finding: GraalVM's standard `--pgo-instrument` flag (build an instrumented image,
     run it to collect `default.iprof`, rebuild with `--pgo`) hits a genuine GraalVM 25.0.3
     native-image compiler crash on this project, 100% reproducible:
     ```
     Fatal error: jdk.graal.compiler.debug.GraalError: mismatched definition: r10|QWORD != r10|QWORD[.]
       at lir instruction: ... SubstrateAMD64DirectCallOp ... callTarget: HostedMethod<InstrumentationData$ProfilingRuntimeCalls.allocateCountersMemory ...>
       at method: void com.oracle.svm.core.foreign.UpcallStubsHolder.upcallLow_I_V_...(int)  [entry point]
       at jdk.graal.compiler.lir.dfa.RegStackValueSet.guaranteeEquals(RegStackValueSet.java:139)
     ```
     — a LIR register-allocator assertion inside the FFM upcall stub `tamboui-panama-backend`
     registers (the build log's own "1 upcalls registered for foreign access" line, every build), most
     plausibly `--pgo-instrument`'s counter-injection colliding with that upcall trampoline's register
     conventions. This is a GraalVM compiler-internals bug, not something fixable from application
     code or Maven config. Worked around (not patched) by using `--pgo-sampling` instead — GraalVM's
     own documented lower-overhead alternative for the same purpose (build+run to collect a profile,
     no code instrumentation) — which built and ran cleanly, dumping a working `default.iprof` even
     from a run that only exercised the app's Spring Boot startup path (the same no-console-handle
     limitation as everywhere else in this phase). Feeding that `default.iprof` into `mise run
     native-pgo` produced `Graal compiler: optimization level: 3 ... PGO: user-provided` (up from
     level 2 / `ML-inferred` on the baseline) and a **40.48MB** binary vs. the baseline's 67.7MB — a
     real, working PGO pipeline end to end, confirmed to still reach the identical no-regression
     `BackendException` checkpoint as every other build in this phase. **Caveat, stated plainly**: the
     `default.iprof` behind that number was collected from a run that never reached the actual TUI
     event loop (no real console here), so it profiles Spring context startup and JSON/PM-detection
     code, not the interactive render/key-handling hot paths PGO would matter most for. The size drop
     is real but likely mostly reachability/dead-code-elimination effects of a narrow profile, not a
     verified interactive-workload speedup. A human re-running `native-instrument`'s exe through the
     full PLAN.md §10.5 walkthrough and regenerating `default.iprof` before a release `native-pgo`
     build is the recommended way to get a profile that actually covers the UI hot paths — documented
     in `README.md`.
- (add more here)
