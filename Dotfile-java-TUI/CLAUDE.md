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

- Current phase: **phase-05 (implementation complete; interactive Definition of
  Done rows pending a human manual run — see below)** (next: verify phase-05
  interactively, then phase-06)
- Completed phases: phase-01, phase-02, phase-03, phase-04
- Phase 5: `mvn compile`/`test` green, Spring context wires end-to-end (every
  bean incl. `TuiApp` resolves). The interactive DoD rows (focus, spinner,
  resize, popup, quit) could NOT be verified from the agent session — no
  `tmux` on this Windows machine, and a background-launched process here has
  no real console handle (`BackendException: Failed to get input console
  mode`), matching Phase 1's own precedent that this app's manual test runs
  in a real Windows Terminal window. **Please run `mise run dev` yourself and
  confirm the checklist in `plan/phase-05-state-loop.md`** before phase-06
  starts. Deviations (sealed `Popup` subtypes moved into `base/` — unnamed-
  module same-package rule; `LazygitLayout` side column is a `.min(24)` floor
  without an upper clamp; new `service/PlatformQueryService` for async PM
  detection) are logged in FEATURE-PARITY.md.
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
