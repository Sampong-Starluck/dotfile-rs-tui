# Phase 13 — installed-list update check + fuzzy filter

**Status:** implemented; interactive verification pending a human run. Requested 2026-08-01,
scoped and built same-day against real captured `winget upgrade` output (see §13.1).

**This is a net-new improvement, not a port** — same situation as Phase 12. Confirmed by
grepping the whole Rust tree (`../src/**`) for `run_list_installed`, `upgrade`, `outdated`,
`update`, `new_version`: the original app has zero per-package "is a newer version available"
concept. The only "upgrade" concept it has is the static `PmCommand` table (`models/
package_manager.rs`) of whole-system upgrade shell commands (`apt upgrade`, `winget upgrade
--all`, `brew upgrade`, …) — already fully ported as `PmCommand`/`CommandsView`, unrelated to
this phase. Rule #1/#2 (Rust = behavior truth) do not apply to the update-check parsing itself;
they still apply to everything else (state/controller/view conventions, DRY, Lombok rules).

## Goal (as requested)

On the Installed-apps list:
1. **Update check.** When the user lists installed apps, also check for available updates. Where
   an update exists, show the new version in a separate **"New Version"** column. The existing
   version column keeps showing the **installed** version — never overwritten.
2. **Fuzzy filter.** After the installed list loads, let the user type to quickly narrow it down
   (fuzzy match, not just exact substring).

## §13.1 Real captured format (2026-08-01, human-run `winget upgrade`)

```
Name                             Id                                  Version                       Available                     Source
---------------------------------------------------------------------------------------------------------------------------------------
DBeaver 26.1.2 (current user)    DBeaver.DBeaver.Community           26.1.2                        26.1.3                        winget
Deno                             DenoLand.Deno                       2.9.0                         2.9.4                         winget
...
16 upgrades available.
```

Same tabular shape as `winget search`/`winget list` (already parsed by `parseWingetSearch`),
with one extra column — "Available" — between Version and Source. The trailing "N upgrades
available." summary line is short enough to always fail the same length guard that already
filters `parseWingetSearch`'s own short/junk lines, so no extra special-casing was needed.
Verified against the pasted output, not assumed (`OutputParsersTest.
wingetUpgradeParsesRealCapturedOutputIntoIdToAvailableVersionMap` uses it verbatim).

## Design decisions

- **Manager scope: winget only for now** (explicit choice — see the DoD). Every other manager's
  `upgradeListCommand` is empty; `PackageQueryServiceImp.checkUpdates` returns an empty map
  without spawning a process when a manager has no known command, matching Phase 12's
  no-data-degrades-cleanly precedent. Extending to apt/brew/choco/pacman/dnf later just needs
  each manager's own real captured output and a small parser — the plumbing (command table,
  async query, state field, UI column) is already generic.
- **`SearchService.upgradeListCommand(mgr) -> Optional<Cmd>`**, not a new full command table
  with placeholder entries for unsupported managers — an empty `Optional` is an honest "not
  supported yet" signal `PackageQueryServiceImp` checks before spawning anything, rather than a
  fake command that would silently misparse.
- **`OutputParsers.parseUpgradeOutput(mgr, text) -> Map<String, String>`** (id -> available
  version), not a new `SearchResult`-like record. `SearchResult` is reused for both search and
  list rows already; adding a 4th "available version" component would force every one of its
  ~20 existing call sites to carry a meaningless field for the search/list case. A side-map
  (mirroring how `InstalledState.names` already keeps installed-only auxiliary data outside
  `SearchResult`) merges into the row view by `id` instead.
- **`checkUpdates` fires alongside `listInstalled`**, both triggered from the single existing
  `CatalogActions.refreshInstalled` (the `r` key, the `l` jump, and the one-time startup
  auto-load all already funnel through it) — one trigger point, two independent futures, both
  drained in `TuiApp.catalogTick()`. Simpler than a separate on-demand trigger, and winget's
  `upgrade` call is no heavier than the `list` call already made every refresh.
- **Fuzzy filter is 100% client-side** (`service/FuzzyMatcher`, a simple case-insensitive
  in-order subsequence match — "dbv" matches "DBeaver"), unlike `SearchInputPopup`'s existing
  `/` binding elsewhere in the app, which shells out to the manager's own remote search command.
  There was no prior client-side filter pattern in the codebase to reuse (checked
  `SearchInputPopup`/`SearchController` — confirmed remote-only). Query lives in
  `InstalledState.filterQuery` (a real `TextInputState`, reusing `Toolkit.handleTextInputKey`
  like every other text input in the app) and narrows `InstalledState.visiblePackages()`, a
  derived/pure method — not a separate mutated list — so cursor/select/remove all naturally
  operate on "what's currently visible" by construction.
- **Filter captures the keyboard while editing** (`InstalledState.filtering`): every key except
  Up/Down/Enter/Esc becomes query text, matching how real fuzzy-finders (fzf etc.) behave —
  chosen so typing letters like "d" or "r" (which are also this view's remove/refresh bindings)
  never accidentally triggers an action while the user is mid-query. Esc clears the filter
  entirely (matches every other popup/input's Esc-cancels convention already in this codebase);
  Enter just stops editing, leaving the filter applied.
- **New Version column, not a popup or separate view** — added directly to the existing
  `InstalledView` row/title (reusing the `Sized.fill`/`Sized.length` column-composition pattern
  `ShellInfoView` already established for a fixed-height row above a filling body), since the
  ask was specifically "another column," and the view already had a fixed 2-column layout to
  extend rather than replace.

## §13.2 Post-review additions (2026-08-01, human interactive pass)

The first human run found three things the initial pass missed:
1. **No way to actually trigger an update** — the human correctly pointed out that showing a
   "New Version" column with no corresponding action is only half the feature ("add update
   package function" was the original ask, not just "show available updates"). Added: `u` on the
   Installed view (mirrors `d` for remove) opens a confirm popup and runs the update through the
   exact same `InstallController`/`InstallExecutionService` streaming pipeline install/remove
   already use — no new execution mechanism. New `InstallKind.UPDATE`,
   `InstallCommandService.updateCommand(mgr, pkg)` (a per-manager single-package upgrade command
   table, same shape/rigor as the existing install/remove tables — e.g. `winget upgrade --id
   <pkg> -e`, `apt install --only-upgrade -y <pkg>`), `CommandPlanner.buildUpdateCommands`, and
   `CatalogActions.openUpdateConfirm` (filters the selection down to ids that actually have a
   pending update in `st.installed.updates` before building commands — pressing `u` with a
   non-updatable package selected is a no-op, same as an empty selection elsewhere).
2. **`?` help popup showed only its first line** — a real `DialogElement`/`Responsive`
   interaction bug (root-caused by reading `tamboui-toolkit-0.4.0-sources.jar` directly), fixed
   by wrapping `HelpPopup`'s body in `Sized.fill(...)` — full writeup in FEATURE-PARITY.md.
3. **Long version strings were truncated** — `InstalledView`'s Version/New-Version columns were
   a fixed 16 cols; changed to size from the longest actual value on screen, clamped `[10,40]`.

## Definition of Done (Phase 13)

- [x] Listing installed apps also checks for available updates (winget: real `winget upgrade`
      output parsed into id -> available-version)
- [x] Installed view gets a "New Version" column; the existing version column always shows the
      installed version, never overwritten
- [x] Managers with no known update-check command degrade cleanly (empty column, no process
      spawned, no fabricated data)
- [x] Fuzzy filter (`/` to edit, type to narrow, Enter to confirm, Esc to clear) narrows the
      installed list by name or id; cursor/select/remove all operate on the filtered view
- [x] `u` triggers an actual update for the selected (and update-eligible) packages, streamed
      through the existing install/remove log popup — §13.2
- [x] `mvn compile`/`test` green (62 tests), incl. new `FuzzyMatcherTest`,
      `OutputParsersTest.wingetUpgradeParses...`/`upgradeOutputDegrades...` built from §13.1's
      literal captured output, and `InstallCommandServiceTest.updateCommandBuildsTheWingetLine`
- [x] Two real bugs found on first human interactive pass fixed (`?` help popup, truncated
      versions — §13.2)
- [ ] Behavior verified interactively in Windows Terminal: real `winget upgrade`-eligible
      packages show correct new-version values, filter narrows/clears correctly, `u` actually
      updates a package and the list refreshes afterward — **needs a human run**
