# Phase 12 — MAIN-panel live detail: streaming command log + download progress

**Status:** noted/backlog, not yet started. Captured 2026-07-31 from a user request during
Phase 7 verification; not scoped or designed in depth yet — flesh out when this phase is
actually picked up.

**This is a net-new improvement, not a port.** Unlike phases 1-11, there is no Rust source
behavior to port for this one — `../src/ui/features/app_tab.rs` has no progress-bar/download-size
parsing at all (checked; only streams raw/animated `\r` output straight to a log, per
`drain_stdout_to_log`, ported as-is in `plan/phase-09-async-install.md` §9.3). Rule #1/#2 (Rust
= behavior truth) do not apply here — this phase's behavior source is whatever the real package
manager CLIs actually print, inspected fresh.

## Goal (as requested)

The MAIN panel (right side of the lazygit layout, PLAN.md §5) should show, for the
in-progress/selected action:
- A **detail view** for the current install/remove/search context.
- A **realtime streaming command log** — output of the running package-manager command,
  updating live, so any error is visible as it happens (not just after the fact).
- When a download is in progress: an **animated progress bar** (arrow style, e.g.
  `[======>>    ]`), the **completion percentage**, the **current downloaded size**, and the
  **full app size**.

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

## Definition of Done (Phase 12) — draft, refine before starting

- [ ] MAIN panel shows a live-updating command log during install/remove, sourced from the
      existing Phase 9 event pipeline (no new streaming mechanism unless Phase 9's proves
      insufficient — Rule #1: don't build a second pipe next to a working one)
- [ ] Download progress renders as an arrow-style bar + percentage + `<downloaded> / <total>`
      size, for managers whose stdout actually carries that data (winget confirmed at minimum)
- [ ] Managers without parseable progress data degrade cleanly (no fabricated numbers)
- [ ] Behavior verified interactively in Windows Terminal against a real small-package install
