# Phase 12 — live detail: streaming command log + download progress

**Status:** in progress. Captured 2026-07-31 from a user request during Phase 7 verification;
scoped and started 2026-08-01 against real captured `winget` output (see §12.1).

**This is a net-new improvement, not a port.** Unlike phases 1-11, there is no Rust source
behavior to port for this one — `../src/ui/features/app_tab.rs` has no progress-bar/download-size
parsing at all (checked; only streams raw/animated `\r` output straight to a log, per
`drain_stdout_to_log`, ported as-is in `plan/phase-09-async-install.md` §9.3). Rule #1/#2 (Rust
= behavior truth) do not apply here — this phase's behavior source is whatever the real package
manager CLIs actually print, inspected fresh.

## Goal (as requested)

For the in-progress install/remove action, the UI should show:
- A **detail view** for the current install/remove context.
- A **realtime streaming command log** — output of the running package-manager command,
  updating live, so any error is visible as it happens (not just after the fact).
- When a download is in progress: an **animated progress bar** (arrow style, e.g.
  `[======>>    ]`), the **completion percentage**, the **current downloaded size**, and the
  **full app size**.

(Originally sketched as living in the MAIN panel — see "Design decisions" below for why this
landed in `InstallLogPopup` instead, which already covers the install/remove context.)

## §12.1 Real captured format (2026-08-01, human-run `winget update`/`winget install`)

Two real winget runs were captured (plain-text terminal copy, not a screenshot, per Rule
"verify, don't assume"):

```
Downloading https://builds.dotnet.microsoft.com/dotnet/Runtime/10.0.10/dotnet-runtime-10.0.10-win-x64.exe
  ██████████████████████████████████   29.2 MB / 29.2 MB
Successfully verified installer hash
Starting package install...
  ██████████████████████████████████   100%
Successfully installed
```
```
Downloading https://github.com/jdx/mise/releases/download/v2026.7.15/mise-v2026.7.15-windows-x64.zip
  ██████████████████████████████  38.6 MB / 38.6 MB
Successfully verified installer hash
Extracting archive...
Successfully extracted archive
Starting package install...
Overwriting existing file: C:\Users\...\mise-shim.exe
Command line alias added: "mise-shim"
Command line alias added: "mise"
Successfully installed
```

Findings that drive the design below:
- Two distinct progress-line shapes: a **download** line (`<bar>  <n> <unit> / <n> <unit>`,
  unit ∈ {B,KB,MB,GB}, no `%`) and an **install** line (`<bar>  <percent>%`, no size). Only
  some installers show the install-phase bar at all (the `.exe`/dotnet-runtime dependency did;
  the top-level `.zip`/`Files.msixbundle` extraction in the second run did not — confirms the
  DoD's "managers/installers without parseable progress data degrade cleanly" case is real and
  common, not hypothetical).
- Both line shapes are **already being matched and dropped** by `DecodeUtil.isNoiseLine` (the
  `t.contains("█")`, `"MB /"`-family, and all-digit-`%` checks) before Phase 12 — today's
  install log never showed a raw progress frame at all, dropped as spinner junk. Phase 12's job
  is to intercept those same lines *before* they're discarded and parse them, not to change
  what counts as noise for the plain scrolling log.
- Windows Terminal does not preserve intermediate `\r`-overwritten frames in scrollback, so only
  the completed (100%) frame of each bar was capturable interactively; this is not a problem for
  the design below since it never tries to read fill-level from winget's own bar glyphs (which
  would require guessing a partial-fill character never actually observed) — only the numeric
  size/percent text is parsed, and the app renders its own arrow-style bar off that number.

## Design decisions (resolving the "once reached" questions below)

- **Popup, not MAIN panel.** PLAN.md §5 fixes `InstallLogPopup` as part of the sealed `Popup`
  set and states the design deletes cross-view focus bookkeeping ("Simpler = correct"). Moving
  the live log into the MAIN panel would re-litigate that fixed decision and complicate Esc
  semantics for no real gain — the popup is already modal and already streams live. Phase 12
  **supplements** `InstallLogPopup` (detail line + progress bar above the log) rather than
  replacing it or moving it. The phase file's original title/DoD said "MAIN panel"; corrected
  here to match §5.
- **New `ProgressBar` component** (`ui/component/ProgressBar.java`), consistent with the
  existing `Panels`/`Lists`/`Logs`/`Popups` pattern (PLAN.md §4 DRY rule): arrow-style
  `[======>>    ]` bar rendered from a parsed percent, not from winget's own bar glyphs.
- **No second streaming pipe.** Reuses the exact Phase 9 pipeline
  (`InstallExecutionServiceImp.drainStdout` → `InstallLogEvent`/new `InstallProgressEvent` →
  `ApplicationEventPublisher` → `InstallLogBridge` → per-frame drain in `TuiApp`). Progress is
  "latest value wins" (a volatile field), not queued like the log lines — there's no reason to
  replay every intermediate animation frame.

## Relationship to Phase 9

Phase 9 (`plan/phase-09-async-install.md`) already builds the streaming pipe: `InstallLogEvent` →
`ApplicationEventPublisher` → `InstallLogBridge` → `ConcurrentLinkedQueue<String>` drained into
`st.installLog` per frame, currently surfaced through the modal `InstallLogPopup` (§9.4). This
phase's job is to decide, once reached:
- Whether the live log/detail moves into the MAIN panel itself (replacing or supplementing
  `InstallLogPopup`), and what that means for focus/Esc semantics (§5's "Esc from any main view
  returns to APPS/Sections deterministically" rule).
- How to parse a progress percentage + downloaded/total size out of each manager's real stdout.
  `winget install` is the concrete case to start from (its own `\r`-animated line already carries
  a percentage and a `<downloaded> / <total>` size pair — verify the exact current format against
  a live `winget install` run, don't assume). Other managers (choco, apt, scoop, brew…) may not
  expose comparable data — decide the no-data fallback (e.g. spinner-only) then.
- Whether the arrow-style bar is a new `ui/component/` (e.g. `ProgressBar`), consistent with the
  existing `Panels`/`Lists`/`Logs`/`Popups` component pattern (PLAN.md §4 DRY rule).

## Definition of Done (Phase 12)

- [x] `InstallLogPopup` shows a detail line (target app name(s), via the already-computed
      `CommandPlanner.displayNames`) + a live-updating command log during install/remove,
      sourced from the existing Phase 9 event pipeline (no new streaming mechanism)
- [x] Download progress renders as an arrow-style bar + percentage + `<downloaded> / <total>`
      size, for managers/phases whose stdout actually carries that data (winget's download-phase
      line confirmed at minimum)
- [x] Managers/phases without parseable progress data degrade cleanly (bar hidden, no fabricated
      numbers) — confirmed real via §12.1's second capture (zip/msixbundle install phase)
- [x] `mvn compile`/`test` green, incl. new `ProgressLineParserTest` cases built from §12.1's
      literal captured lines
- [ ] Behavior verified interactively in Windows Terminal against a real small-package install
      (winget at minimum) — **needs a human run**
