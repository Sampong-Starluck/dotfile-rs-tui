# dotfile-java-tui — implementer instructions

You are implementing a phased migration plan. **Do not improvise scope.**

## Before writing ANY code

1. Read `PLAN.md` fully (rules #0/#1, §4 conventions, §4a Lombok, §5 UI design, §5a state, §5b 200 ms rule).
2. Read the ONE phase file you were asked to implement in `plan/`.
3. Check `## STATUS` below — never start a phase whose predecessor isn't done.

## Hard rules (repeated because they get skipped)

- Implement exactly ONE phase per session, in order. Finish = every
  "Definition of Done" checkbox verified, then tick the matching rows in
  `FEATURE-PARITY.md` and update `## STATUS` below.
- TamboUI names: if a class/method from the plan doesn't compile, grep the
  `api-*.txt` files (generated in phase 1) for the real name — never invent.
- Rust sources at `../src/**` = behavior truth (commands, parsers, paths,
  state transitions). NOT UI structure, NOT hacks.
- Services: interface in `service/`, `@Service` class `<Name>Imp` in
  `service/implementation/`. Beans: `@Slf4j @RequiredArgsConstructor`.
  No `@Data`/`@Setter`/`@SneakyThrows`. No Lombok on records/state.
- All UI = fluent Toolkit DSL `Element` trees. State only in `state/`.
  No `System.out` (except `ExternalRunner`). Log to `debug.log` via slf4j.
- Startup: nothing synchronous > 200 ms — lazy + spinner instead.

## Commands

```powershell
mise install        # toolchain (GraalVM 25 + Maven)
mise run dev        # run the TUI
mise run test       # unit tests — must be green before a phase is "done"
mise run build      # fat jar
```

Compile after every file: `mise exec -- mvn -q compile`.

## STATUS

- Current phase: **phase-11 (native-image build succeeds; interactive
  acceptance-run row pending a human manual run — see below)** (next: verify
  phase-11's native exe interactively; phase-11 is the last phase in the
  main sequence — phase-12 is an unscoped backlog item, not next in order)
- Completed phases: phase-01 through phase-10 (all Definition of Done rows,
  incl. the interactive ones, confirmed by a human running `mise run dev`/
  the fat jar in a real Windows Terminal window)
- Phase 11: `mvn compile`/`test` green (50 tests, unchanged — no new business
  logic). `mise run native` (`mvn -Pnative native:compile`) succeeds:
  `target/dotfile-java-tui.exe`, 67.7MB, ~3 min build. Two real
  native-image-specific issues were hit and fixed (recorded in detail in
  FEATURE-PARITY.md): (1) `tamboui-tui-0.4.0.jar`'s built-in key-binding-set
  `.properties` resources ship with no native-image metadata of their own —
  fixed with a new `config/NativeHints.TamboUiResources`
  (`RuntimeHintsRegistrar` via `@ImportRuntimeHints`) registering
  `dev/tamboui/tui/bindings/*.properties`; (2) `WindowsTerminal`'s FFM code
  closes a shared `Arena`, which native-image disables unless
  `-H:+SharedArenaSupport` is passed — added to the `native` Maven profile's
  buildArgs, **superseding** PLAN.md §11.2's speculative
  `-H:+ForeignAPISupport` guess (not needed — `tamboui-panama-backend` ships
  its own `reachability-metadata.json` covering its 21 FFM downcalls + 1
  upcall already). `config/NativeHints` also carries
  `@RegisterReflectionForBinding` for the six Jackson-mapped records per
  §11.3. After both fixes, launching `target\dotfile-java-tui.exe` from this
  agent session reaches `BackendFactory.create()` and fails with the same
  class of `BackendException` (no real console handle) every prior phase's
  agent-launched run has hit — not a regression, genuine parity with the fat
  jar's own startup path, but **not** full interactive verification.
  Post-review follow-up on the build log's own warnings (full detail in
  FEATURE-PARITY.md): removed the unused `spring-boot-starter-actuator`
  dependency (source of a `DynamicProxyConfigurationResources` deprecation
  warning via transitive `micrometer-core`; zero actuator/micrometer usage
  anywhere in the codebase, not in PLAN.md §3's dependency table either —
  dead weight, not a warning worth working around); paired
  `-H:+SharedArenaSupport` with `-H:+UnlockExperimentalVMOptions` proactively
  (GraalVM warns a future release will require it); implemented a full PGO
  workflow (`mise run native-instrument` → exercise the exe → `mise run
  native-pgo`), parametrizing the one `native` Maven profile via `-D`
  properties rather than separate profile ids (separate ids broke Spring
  Boot's own built-in `native`-named profile inheritance — same root cause
  class as the mainClass issue below). GraalVM's standard `--pgo-instrument`
  flag hit a **genuine, 100%-reproducible GraalVM 25.0.3 native-image
  compiler crash** (a LIR register-allocator assertion inside the FFM upcall
  stub `tamboui-panama-backend` registers) — not fixable from application
  code; worked around with `--pgo-sampling` (GraalVM's own lower-overhead
  alternative), which built, ran, and produced a working `default.iprof`.
  Feeding that into `native-pgo` produced a real PGO build (`PGO:
  user-provided`, optimization level 3, 40.48MB vs. the baseline's 67.7MB),
  confirmed to reach the same no-regression checkpoint — but the profile
  behind it only covers the app's startup path (same no-console-handle
  limitation), not the interactive TUI hot loop, so the size drop is real
  but not a verified interactive-workload speedup; a human re-collecting
  `default.iprof` from the full §10.5 walkthrough before a release
  `native-pgo` build is recommended (documented in `README.md`).
  **Please run `target\dotfile-java-tui.exe` yourself in a real Windows
  Terminal and confirm the acceptance checklist in
  `plan/phase-11-native-image.md` §11.4 / PLAN.md §10.5** to close out
  phase-11's remaining Definition of Done row.
- Phase 10: `mvn compile`/`test` green (50 tests, unchanged — this phase adds
  no new business logic to unit-test, only UI chrome + packaging).
  `mise run build` produces `target/dotfile-java-tui-0.1.0.jar`; running it
  directly (`java --enable-native-access=ALL-UNNAMED -jar
  target\dotfile-java-tui-0.1.0.jar`) reached the same `BackendException:
  Failed to get input console mode` recorded since Phase 5 — not a
  regression (this agent session still has no real console handle). New:
  `ui/Bindings` as the single source for the bottom hint bar + `?` help
  popup (both previously drifted independently — see FEATURE-PARITY.md),
  `README.md`, `dotfile.cmd` launcher. Convention audit (Lombok, `…Imp`
  encapsulation, `System.out` scope) grepped clean. Deviations (mouse
  row-click/wheel-scroll extras skipped — `Lists.selectable` has no
  per-row `renderedArea()` to hit-test against without a rendering-model
  redesign; `HelpPopup`'s content height is left natural, not
  percentage-clamped, since `DialogElement` has no such API) are logged in
  FEATURE-PARITY.md. A human has since run `mise run dev` in a real Windows
  Terminal and confirmed the checklist in `plan/phase-10-polish-packaging.md`
  §10.5. Phase 10 is fully done.
- Phase 9: `mvn compile`/`test` green (50 tests, unchanged — this phase's DoD
  is entirely live process-spawning/interactive behavior, not new unit
  tests). `mise exec -- mvn -q spring-boot:run` reached the same
  `ToolkitApp.run()` → backend-creation point as every prior phase (all new
  beans — `InstallExecutionService`/`InstallExecutionServiceImp`,
  `InstallLogBridge`, `TuiApp`'s new `systemService`/`installExecutionService`/
  `installLogBridge` deps — resolve cleanly; `TuiApp.onStart()` builds
  `InstallController` and wires it into every catalog-adjacent controller)
  and failed only at the same `BackendException: Failed to get input console
  mode` recorded since Phase 5 — not a regression. **Please run `mise run
  dev` yourself (with real package managers installed — winget at minimum,
  choco for the suspend/resume row) and confirm the checklist in
  `plan/phase-09-async-install.md`** before phase-10 starts. Deviations
  (`TuiApp` now overrides `ToolkitApp.run()` with a close-and-recreate loop
  around the `ToolkitRunner` since the 0.4.0 API has no pause/resume, handing
  off to the new `ui/ExternalRunner` between lifecycles once the terminal is
  already restored; `InstallLogBridge`'s render-waker is setter-injected by
  `TuiApp.onStart()` instead of constructor-injected, to avoid a `TuiApp <->
  InstallLogBridge` circular bean dependency; `SudoPopup`'s password field
  uses the toolkit's real `TextInputState` instead of the Phase-7 stub's raw
  `StringBuilder`, reusing `Toolkit.handleTextInputKey` like
  `CustomInputPopup`/`SearchInputPopup` already do) are logged in
  FEATURE-PARITY.md.
- Phase 8: `mvn compile`/`test` green (50 tests, incl. 6 new
  `ScriptServiceImpTest` cases). `mise exec -- mvn -q spring-boot:run` reached
  `ToolkitApp.run()` (all new beans — `ScriptService`/`ScriptServiceImp`,
  `TuiApp`'s new `scriptService` dep, `ScriptsController` — resolve cleanly;
  `TuiApp.onStart()` runs) and failed only at the same `BackendException:
  Failed to get input console mode` recorded since Phase 5 — not a
  regression. A human has since run `mise run dev` in a real Windows Terminal
  and confirmed every checklist row in `plan/phase-08-script-tab.md`
  (Shells panel listing, deploy, idempotent re-deploy, undeploy, set/clear
  primary shell). Deviations
  (`model/ShellStatus` widened with pure `binary`/`profilePath`/`sourceHint`
  fields, computed once in `ScriptServiceImp.loadShellStatuses()`, so
  `ShellInfoView` stays zero-service-call; lazy shells load lives in a new
  `TuiApp.scriptsTick()` mirroring `catalogTick()`, not inline in
  `ShellsView.render()` like the Rust `script_render`; `ScriptsState` gained
  `explicitPrimaryShell` alongside the effective `primaryShell` so the view
  can render the three-way primary-line distinction from state alone) are
  logged in FEATURE-PARITY.md.
- Phase 7: `mvn compile`/`test` green (44 tests, incl. 5 new `CommandPlannerTest`
  cases). A human ran `mise run dev` in a real Windows Terminal and walked
  every interactive DoD row in `plan/phase-07-catalog-search-installed.md`.
  That first live run surfaced 3 real, pre-existing bugs (not introduced by
  Phase 7's own diff) — a panel-collapse layout bug (`column(panel).fill()`
  wrappers re-derived a content-sized constraint for their single child
  instead of stretching it; fixed with new `ui/component/Sized`), a
  catalog/installed startup race (`st.catalog.apps` filtered against
  `activeBinary()`'s `"unknown"` placeholder before async manager detection
  landed, permanently empty; fixed by gating on `!st.platform.detecting`),
  and a `DialogElement` popup-width bug in the TamboUI framework itself
  (worked around in `ui/component/Popups`). All three fixed, re-verified
  green. Full root-cause writeup in FEATURE-PARITY.md's Phase 7 deviations.
  Other deviations (`AppState.pendingFocus`/`requestFocus` for
  controller-requested focus jumps applied by `TuiApp` before `syncFocus()`;
  `PackageQueryService` stub wiring the future/drain plumbing ahead of Phase 9's
  real process spawning — confirmed expected/by-design after human review, not
  a bug; `ConfirmActionPopup`/`Popups.confirm` gained a `warning`
  line; `SearchInputPopup`/`CustomInputPopup` switched from `StringBuilder` to the
  toolkit's real `TextInputState`; `ui/feature/catalog/CatalogActions` as the
  single shared home for confirm/search/custom popup construction + installed
  refresh, reused by all four catalog-adjacent controllers) are logged in
  FEATURE-PARITY.md.
- Backlog (not a phase in sequence): `plan/phase-12-improvement.md` — MAIN-panel
  live detail (streaming command log + download progress bar), requested by the
  user during Phase 7 verification, explicitly deferred as a post-parity
  improvement. Not scoped; revisit after Phase 11.
- Phase 6: `mvn compile`/`test` green; `mise run dev` reaches `ToolkitApp.run()`
  (all beans, incl. the new `ui/component/Responsive` and its use in
  `ManagersView`/`CommandsView`, resolve cleanly) and, when launched from the
  agent session (no real console handle available there), fails only at the
  same `BackendException: Failed to get input console mode` recorded for
  Phase 5 — not a regression. A human has since run `mise run dev` in a real
  Windows Terminal and confirmed every checklist row in
  `plan/phase-06-status-managers.md` (and `plan/phase-05-state-loop.md`'s).
  Deviations (a new `ui/component/Responsive` element for width-dependent
  degrade rules since the fluent tree is built before Cassowary layout runs;
  the Commands table's `[n/total]` scroll indicator lives in the panel body,
  not the MAIN panel's border title, for the same reason;
  `FeatureView.title(AppState)` added so MAIN panels can set a dynamic
  border title) are logged in FEATURE-PARITY.md.
- Phase 5: `mvn compile`/`test` green, Spring context wires end-to-end (every
  bean incl. `TuiApp` resolves). The interactive DoD rows (focus, spinner,
  resize, popup, quit) could not be driven from the agent session — no
  `tmux` on this Windows machine, and a background-launched process here has
  no real console handle (`BackendException: Failed to get input console
  mode`), matching Phase 1's own precedent that this app's manual test runs
  in a real Windows Terminal window — since confirmed by a human run.
  Deviations (sealed `Popup` subtypes moved into `base/` — unnamed-module
  same-package rule; `LazygitLayout` side column is a `.min(24)` floor without
  an upper clamp; new `service/PlatformQueryService` for async PM detection)
  are logged in FEATURE-PARITY.md.
- Blockers/deviations: added `spring-boot-starter-jackson` to `pom.xml` (not
  pulled in by `spring-boot-starter` alone) to get the Boot-managed Jackson 3
  `ObjectMapper`; real package is `tools.jackson.databind.ObjectMapper` /
  `tools.jackson.core.type.TypeReference`, group `tools.jackson.core`.
  Phase 3: `PathService.which` on Windows must check existence via
  `LinkOption.NOFOLLOW_LINKS` rather than `Files.isRegularFile` — WindowsApps
  App Execution Alias reparse points (e.g. `winget.exe`) fail the follow-links
  stat, which silently broke winget detection (see FEATURE-PARITY.md).
  Phase 4: `parse_apt_search` in the Rust source has a pre-existing bug
  (version = first whitespace token after the first `/`, which is actually
  the suite codename, not the version) — ported byte-perfect per Rule #1,
  NOT the phase-04-parsers.md fixture answer for that case, which does not
  match the real Rust behavior (verified against `splitn(2,'/')` semantics;
  see FEATURE-PARITY.md deviations). Live smoke test against real
  `winget search git` on this machine returned 319 parsed rows.
