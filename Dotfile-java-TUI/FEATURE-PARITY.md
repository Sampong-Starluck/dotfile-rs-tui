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
| `app_tab.rs::build_commands/build_remove_commands/selected_display_names` | `service/CommandPlanner` | ☐ |
| `src/service/script_service.rs` (all fs/profile/config logic) | `service/ScriptService` | ☐ |
| `src/logging.rs` (file-only logging) | `logback-spring.xml` → `debug.log` | ☐ |

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
| catalog sections + apps list, `[✓]`/`[I]`, badge | `feature/catalog/` SectionsPanel + AppsView | ☐ |
| custom package input | `CustomInputPopup` | ☐ |
| search input + results (3-col, loading, empty) | `feature/search/` SearchInputPopup + SearchResultsView | ☐ |
| installed view (red remove styling, refresh) | `feature/installed/InstalledView` | ☐ |
| install/remove decision tree | `feature/install/InstallController` | ☐ |
| (new) explicit confirm before running commands | `ConfirmActionPopup` (lazygit-style) | ☐ |
| sudo password modal | `SudoPopup` (masked input added) | ☐ |
| install modal: colored log, autoscroll, stdin input | `InstallLogPopup` + `LogView` | ☐ |
| scripts tab (shells list, info, log, all keys) | `feature/scripts/` ShellsPanel + ShellInfoView + ScriptsController | ☐ |

## Async & external (Phase 9 — Spring-native)

| Rust behavior | Java target | Done |
|---|---|---|
| `run_search` thread + mpsc | `PackageQueryService.search` `@Async` → `CompletableFuture`, polled per frame | ☐ |
| `run_list_installed` | `PackageQueryService.listInstalled` | ☐ |
| streamed install/remove + stdin relay + sudo -S | `InstallExecutionService.runStreaming` + `InstallLogEvent` + `InstallLogBridge` | ☐ |
| `drain_stdout_to_log` (\r\n split, ansi strip, noise filter) | `InstallExecutionService.drainStdout` | ☐ |
| `main.rs::run_in_terminal` suspend/inheritIO/restore/reset | `ui/ExternalRunner` | ☐ |

## Chrome & packaging (Phases 10–11)

| Item | Target | Done |
|---|---|---|
| status-bar hints + help overlay (single source) | `ui/Bindings` → `KeyHintBar` + `HelpPopup` | ☐ |
| mouse (clicks, wheel) | works / unsupported note: ______ | ☐ |
| fat jar + launcher + README | Phase 10 | ☐ |
| GraalVM native-image | Phase 11 — built / blocked because: ______ | ☐ |

## Resources & interop

| Item | Target | Done |
|---|---|---|
| `src/json/apps.json`, `src/json/shells.json` | `resources/data/` | ☐ |
| `src/scripts/**` (5 profiles) | `resources/scripts/**` | ☐ |
| `%APPDATA%\dotfile-rs` + `config.json` snake_case kept Rust-compatible | `ScriptService` + `DotfileConfig` | ☐ |

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
  from this session — there is no `tmux` on this Windows machine, and a
  background-launched process here has no real console handle
  (`BackendException: Failed to get input console mode`, the same failure
  any TUI would hit launched this way). This matches Phase 1's own
  precedent: the Definition of Done's focus/spinner/resize/popup/quit items
  need a human to run `mise run dev` in an actual Windows Terminal window.
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
  launched from this session and reached `ToolkitApp.run()` (all beans,
  incl. the new `Responsive`-based views, resolve cleanly), failing only at
  the same `BackendException: Failed to get input console mode` recorded in
  Phase 5 — not a regression. The interactive DoD rows (spinner timing,
  cursor/`●` movement, table scroll/clamp, hint-bar focus updates) need a
  human to run `mise run dev` in an actual Windows Terminal window.
- (add more here)
