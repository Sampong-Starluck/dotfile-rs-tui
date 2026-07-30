# Phase 6 — Status panel, Managers panel, Commands main view

**Goal:** the first three real features under `ui/feature/`. Proves the
fluent component factories (`Panels.framed`, `Lists.selectable`,
`HintBar.of`) on real data.

Behavior source: `../src/ui/features/home_tab.rs` (what to show), lazygit
layout (how to show it).

## 6.1 `feature/status/` — StatusPanel (view only, no controller actions)

- Renders inside `Panels.framed(1, "Status", …)`, height 3 → one line:
  `"<os> · <manager label>"` — OS cyan bold, `·` darkGray, manager yellow
  bold; `"no manager"` darkGray when none detected.
- When STATUS is focused, `mainView = COMMANDS` (same as MANAGERS) — the
  panel is informational; its controller consumes nothing (returns false).

## 6.2 `feature/managers/` — ManagersPanel + ManagersController

**View** (`ManagersPanel implements FeatureView`): `Lists.selectable` of
`st.packageManagers`; row = `marker + label`:
- `"● "` green for the **active** manager (`st.selectedPm`), `"  "` otherwise;
- label = `pm.label()`, or `pm.binary()` when panel width < 18;
- append ` — <pm.description()>` dim when width allows (≥ 30).
- `st.platform.detecting` → `row(spinner(), text(" detecting…").dim())`
  (lazy detection per PLAN.md §5b — never block the first frame on PATH probes).
- Detection done + empty list → `"no package managers detected"` dim.
- Title `"[2]─Package managers"`.

**Controller** (`ManagersController implements KeyController`):

```
Up/Down (j/k) → move st.managersCursor (clamped)
Enter / Space → st.activateManager(st.managersCursor)      // resets catalog state, §5.2
```
When MANAGERS (or STATUS) has focus, `st.mainView = COMMANDS` — set this in
the controller on focus-relevant keys AND defaulted by `TuiApp` when focus
changes to these panels (one switch in `TuiApp.focusChanged`).

## 6.3 `feature/managers/CommandsView` — main panel content (`MainView.COMMANDS`)

Port the table logic of `home_tab.rs::render_command_table` into the lazygit
main panel; show commands for the manager **under the cursor** (so moving the
cursor previews without activating — lazygit preview behavior), falling back
to the active manager when the cursor panel isn't MANAGERS:

- Title: `"Commands — <label>"` + `"  [<scroll+1>/<total>]"` when scrollable.
- Two columns: command yellow, description white; header row cyan bold;
  `cmdWidth = max(45% of inner, 15)`; description hidden when its width < 15.
- Scroll state `st.commandScroll`, clamped to `total - visibleRows`;
  PgUp/PgDn (and J/K capitals) scroll when MANAGERS/STATUS focused.
- Area too small (w<20 or h<4) → `"terminal too small"` dim.
- Use the toolkit's fluent `table()` factory if it supports per-cell styles
  + a header row; otherwise compose a `column()` of two-segment `row()`s
  padded with `UiText.padRight` — pick ONE approach after checking
  `api-toolkit.txt`, don't build both.

## 6.4 Key hints (extend `Bindings`)

`Bindings.for(st)` returns, when focus = MANAGERS:
`enter: use manager · j/k: move · pgup/pgdn: scroll commands · 1-4: jump · ?: help · q: quit`
(rendered by `HintBar.of`; wording short like lazygit).

## Definition of Done (Phase 6)

- [ ] App's first frame is instant; panel `[2]` shows the detecting spinner briefly, then real managers (verify timing lines in `debug.log`) — **pending human run in Windows Terminal, see below**
- [ ] Status shows `Windows · winget` (or the active manager) live — **pending human run**
- [ ] Managers panel lists real detected managers; cursor moves; `●` follows Enter — **pending human run**
- [ ] Main panel previews the cursor manager's command table; activating with Enter keeps state consistent (catalog reset happens — visible in Phase 7) — **pending human run**
- [ ] Commands table scrolls and clamps correctly; narrow/short windows degrade per spec — **pending human run**
- [ ] Hint bar updates when focus enters these panels — **pending human run**

**Verification note:** `mvn compile`/`test` are green. `mise run dev` was launched from this
session to confirm Spring wiring: it reaches `ToolkitApp.run()` (all beans, including the new
`Responsive` component usage inside `ManagersView`/`CommandsView`, resolve cleanly) and fails only
at backend creation (`BackendException: Failed to get input console mode`) — the same
no-real-console-handle limitation recorded in Phase 5, not a regression from this phase. The
interactive rows above need a human to run `mise run dev` in an actual Windows Terminal window.
