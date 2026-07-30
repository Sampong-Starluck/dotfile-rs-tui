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

- Current phase: **phase-02 (complete)** (next: phase-03)
- Completed phases: phase-01, phase-02
- Blockers/deviations: added `spring-boot-starter-jackson` to `pom.xml` (not
  pulled in by `spring-boot-starter` alone) to get the Boot-managed Jackson 3
  `ObjectMapper`; real package is `tools.jackson.databind.ObjectMapper` /
  `tools.jackson.core.type.TypeReference`, group `tools.jackson.core`.
