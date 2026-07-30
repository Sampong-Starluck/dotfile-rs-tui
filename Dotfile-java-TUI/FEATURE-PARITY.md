# Feature parity checklist — Rust behavior → Java target

Tick each row when its **behavior** is ported AND verified. The UI is
redesigned to lazygit style (PLAN.md §5), so UI rows map Rust *functionality*
to its new home, not its old look. Paths relative to the Rust root (`../`).

## Toolchain

| Item | Target | Done |
|---|---|---|
| mise-managed GraalVM 25 + Maven | `mise.toml` | ☑ |
| Spring Boot 4.1.0, no web, banner off | `pom.xml`, `application.yml` | ☑ |
| Virtual threads + @EnableAsync | `config/AsyncConfig`, `spring.threads.virtual.enabled` | ☑ |
| TamboUI 0.4.0 **Toolkit DSL (fluent API)** + Panama FFM backend (Windows verified) | `tamboui-toolkit` in pom + Phase 1 smoke test | ☑ |
| All UI written as fluent `Element` trees (no immediate-mode calls outside `Popups.overlay`) | `ui/**` | ◐ |
| Every service = interface in `service/` + `<Name>Imp` in `service/implementation/` (only Imp has `@Service`) | all phases | ◐ |
| State management: composed `state/` classes, unidirectional flow, no state in views/components | `state/*` | ◐ |
| 200 ms rule: no synchronous startup step > 200 ms; slow loads lazy with spinner (PM detection async) | Phase 5/6/10 audit | ◐ |
| Lombok scoped rules: `@Slf4j` + `@RequiredArgsConstructor` mandatory on beans; no `@Data`/`@Setter`/`@SneakyThrows`; none on records/state | PLAN.md §4a audit | ☑ |
| MapStruct: evaluated, not used (no DTO layer) — decision recorded; revisit only if a mapping layer appears | — | ☑ |
| `--enable-native-access=ALL-UNNAMED` everywhere (mise env, boot plugin, launcher) | Phase 1/10 | ☑ |

## Models & config (Phase 2)

| Rust source | Java target | Done |
|---|---|---|
| `src/models/os.rs` (types) | `model/OperatingSystem`, `model/LinuxDistro` | ☐ |
| `src/models/package_manager.rs` (binary/label/commands data) | `model/PackageManager`, `model/PmCommand` | ☐ |
| `src/models/apps.rs` | `model/AppSection`, `model/AppEntry` | ☐ |
| `src/models/shell.rs` + `shell_model.rs` | `model/ShellEntry`, `model/ShellsFile` | ☐ |
| `SearchResult`, `ShellStatus`, `DotfileConfig` (snake_case JSON) | `model/*` | ☐ |
| Rust `TabModel`/`AppFocus` | **replaced** by `PanelId` + `MainView` + sealed `Popup` | ☐ |
| — | `config/AppProperties` (data dir name, default `dotfile-rs`) | ☐ |

## Services (Phases 3, 4, 8)

| Rust source | Java target | Done |
|---|---|---|
| `os.rs::detect` + `/etc/os-release` | `service/OsService` | ☐ |
| `which` crate | `service/PathService.which` (PATHEXT-aware) | ☐ |
| `package_manager.rs::detect/candidates_for` | `service/PackageManagerService` | ☐ |
| `src/service/system_service.rs` | `service/SystemService` | ☐ |
| `src/service/install_service.rs` | `service/InstallCommandService` | ☐ |
| `src/service/app_service.rs` | `service/AppCatalogService` | ☐ |
| `search_service.rs` command tables + hint | `service/SearchService` | ☐ |
| `src/utils/decode_util.rs` | `service/DecodeUtil` | ☐ |
| `src/utils/text_util.rs` | `service/TextUtil` | ☐ |
| `search_service.rs` 9 search + 8 list parsers | `service/OutputParsers` | ☐ |
| `app_tab.rs::build_commands/build_remove_commands/selected_display_names` | `service/CommandPlanner` | ☐ |
| `src/service/script_service.rs` (all fs/profile/config logic) | `service/ScriptService` | ☐ |
| `src/logging.rs` (file-only logging) | `logback-spring.xml` → `debug.log` | ☐ |

## UI shell (Phase 5)

| Rust behavior | Java target (lazygit design) | Done |
|---|---|---|
| `src/app.rs::App` state + reset logic | `state/AppState` (incl. `activateManager` reset) | ☐ |
| `src/main.rs` loop: press-only keys, help gating, quit, resize | `ui/TuiApp` | ☐ |
| Tab switching | toolkit focus system: `.id().focusable()`, Tab/Shift-Tab/click native, `1-4` programmatic, Enter/Esc into MAIN | ☐ |
| loading spinner | toolkit built-in animated `spinner()` | ☐ |
| `src/ui/layout/layout.rs` | `ui/layout/LazygitLayout` (fluent row/column composition) | ☐ |
| shared panel/list/input/log/popup drawing | `ui/component/*` factories (Panels, Lists, Inputs, Logs, Popups, HintBar) | ☐ |
| `base/` contracts (FeatureView, KeyController, sealed Popup) | `base/*` | ☐ |
| background completion → re-render | `requestRender` wiring on futures + log bridge | ☐ |

## Features (Phases 6–9)

| Rust behavior | Java target | Done |
|---|---|---|
| home sidebar (PM list) + platform info | `feature/status/StatusPanel` + `feature/managers/ManagersPanel` | ☐ |
| home command cheat-sheet table + scroll | `feature/managers/CommandsView` | ☐ |
| PM picker modal | **deleted** — Managers panel is the picker | ☐ |
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
- (add more here)
