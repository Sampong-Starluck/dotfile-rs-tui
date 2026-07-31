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

- Current phase: **phase-08 (implementation complete; interactive Definition
  of Done rows pending a human manual run — see below)** (next: verify
  phase-08 interactively, then phase-09)
- Completed phases: phase-01 through phase-07 (all Definition of Done rows,
  incl. the interactive ones, confirmed by a human running `mise run dev` in
  a real Windows Terminal window)
- Phase 8: `mvn compile`/`test` green (50 tests, incl. 6 new
  `ScriptServiceImpTest` cases). `mise exec -- mvn -q spring-boot:run` reached
  `ToolkitApp.run()` (all new beans — `ScriptService`/`ScriptServiceImp`,
  `TuiApp`'s new `scriptService` dep, `ScriptsController` — resolve cleanly;
  `TuiApp.onStart()` runs) and failed only at the same `BackendException:
  Failed to get input console mode` recorded since Phase 5 — not a
  regression. **Please run `mise run dev` yourself and confirm the checklist
  in `plan/phase-08-script-tab.md`** before phase-09 starts. Deviations
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
