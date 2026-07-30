# Phase 7 — Catalog, Search, Installed features

**Goal:** the heart of the app: Sections panel, Apps main view, Installed
main view, Search flow (input popup + results view), custom-id popup, and the
lazygit-style confirm popup. Async *execution* stays stubbed until Phase 9,
but all wiring (futures, drains) goes in now.

Behavior source: `../src/ui/features/app_tab.rs` — port its *logic* (selection
rules, `[✓]`/`[I]` markers, command building, decision tree), not its
focus/modal structure.

## 7.1 Per-frame catalog upkeep (in `CatalogController` or a shared
`FrameTick` step invoked by `TuiApp` before render — pick one place, DRY)

1. `st.apps == null` → load + filter:
   `catalog.filterByPlatform(catalog.readAppsJson(), os.osKey(), st.activePackageManager())`.
2. Drain futures (KISS polling):
   - `st.searchFuture != null && isDone()` → `st.searchResults = join()`
     (empty list on exception), `searchLoading=false`, `searchCursor=0`,
     `searchFuture=null`, `st.mainView = SEARCH_RESULTS`.
   - `st.installedFuture` likewise → also rebuild `st.installedSet` from names.
3. Auto-load installed once:
   `!st.installedAutoLoaded && !st.installedLoading` → set flag, call
   `queryService.listInstalled(...)` (Phase 9 makes it real; stub returns a
   completed future with `List.of()` for now).

## 7.2 `feature/catalog/SectionsPanel` (`[3]─Sections`)

`Lists.selectable` of `st.apps` section names; cursor row per component
defaults; when `st.apps` empty → `"no apps for <mgr>"` dim.

**`SectionsController`:**

```
Up/Down     → st.sectionCursor (clamped); st.appCursor = 0
Enter / →   → st.focused = MAIN; st.mainView = APPS
'/'         → st.popup = new SearchInputPopup(new StringBuilder(st.lastQuery))
'c'         → st.popup = new CustomInputPopup(new StringBuilder())
'l'         → st.mainView = INSTALLED; st.focused = MAIN; st.removeMode = true;
              refresh installed (clear lists + trigger query)
'd'         → openConfirmInstall(st)          // §7.6
```

## 7.3 `feature/catalog/AppsView` (`MainView.APPS`)

Shows the apps of the cursor section. Row =
`checkbox + name` where checkbox is `[✓]` (id in `st.selectedIds`) /
`[I]` (installed, green) / `[ ]`; styling matrix ported from
`app_tab.rs::render_app_list`: selected → green bold; installed → green;
cursor via `Lists.selectable` (reversed).

`isInstalled(entry)` port of `entry_installed` (:267): resolve
`entry.platforms[osKey][mgr]` and check `st.installedSet`, fallback
`installedSet.contains(entry.id)`.

Title: `"<section> — <mgr>"` + `"  ✓ <n>"` when `n = selectedIds.size() > 0`.

**`AppsController`** (active when focused == MAIN && mainView == APPS):

```
Up/Down → st.appCursor
Space   → toggle entry.id in st.selectedIds
'/'     → search popup      'c' → custom popup
'l'     → INSTALLED view (as in Sections)
'd'     → openConfirmInstall(st)
Esc / ← → st.focused = SECTIONS
```

## 7.4 Search feature (`feature/search/`)

### `SearchInputPopup` (record implementing `base/Popup`, holds `StringBuilder query`)
`Popups.overlay(…, 50, 20)`, title `"⌕ Search <mgr>"`, `Inputs.line` row +
dim hint `"enter: search · esc: cancel"`.
Controller: printable chars/backspace edit; Enter (non-empty) →
`st.lastQuery = query; st.searchLoading = true; st.popup = null;
st.searchFuture = queryService.search(mgr, query)` (stub returns completed
fake rows this phase); Esc → `st.popup = null`.

### `SearchResultsView` (`MainView.SEARCH_RESULTS`)
Port of `render_search_panel` styling: three columns (id yellow / name / version
dim) sized 40/40/rest of `innerW − 6 − 4`; checkbox `[✓]`/`[I]`/`[ ]`;
loading state → the toolkit's built-in animated `spinner()` +
`"searching…"` yellow title; empty → hint text.
Title: `"Search <mgr> — <n> result(s)"`.

Controller: Up/Down; Space toggles `result.id` in `selectedIds`; `/` reopens
the input popup pre-filled with `st.lastQuery`; `d` → confirm install;
Esc → `st.mainView = APPS` (results kept until next search or manager switch).

## 7.5 Installed feature (`feature/installed/InstalledView`)

Port of `render_installed_panel`: rows `checkbox + name + version`
(verW 16); picked rows **red** (remove semantics): cursor+picked reversed
red, picked red bold. Loading → spinner title; empty →
`"no installed packages — r: refresh"`.
Controller: Up/Down; Space toggle; `r` → clear + re-query; `d` →
openConfirmRemove; Esc → `st.mainView = APPS; st.removeMode = false;
st.selectedIds.clear(); st.focused = SECTIONS`.

## 7.6 Custom-id popup + confirm popup

### `CustomInputPopup`
Same shell as search popup, title `"⌕ Custom package id"`. Enter/Space commit
the trimmed buffer into `st.selectedIds` and close; Esc discards.

### `ConfirmActionPopup` (lazygit-style confirmation — replaces silent execution)
Built by `openConfirmInstall/Remove(st)`:

```java
List<String> commands = commandPlanner.buildInstallCommands(st);   // §7.7
if (commands.isEmpty()) return;              // nothing selected → no-op
st.popup = new ConfirmActionPopup(kind /*INSTALL|REMOVE*/, commands);
```
View: `Popups.confirm` listing the exact command strings (max 10 + overflow),
title `"Install <n> package(s)?"` / `"Remove …?"`, warning line
`"⚠ runs with <mgr>"`.
Controller: `y`/Enter → close popup and hand to `InstallController.start(st,
kind, commands)` (Phase 9; stub logs); `n`/Esc → close.

## 7.7 `service/CommandPlanner` (pure logic extracted for DRY + testability)

Port of `build_commands` / `build_remove_commands` /
`selected_display_names` (`app_tab.rs:1669/:1531/:1817`):

Interface + Imp convention as everywhere:

```java
// service/CommandPlanner.java
public interface CommandPlanner {
    List<String> buildInstallCommands(List<AppSection> apps, Set<String> selectedIds, String mgr);
        // catalog ids via installCommandFor, unknown ids via installCommand(mgr, id)
    List<String> buildRemoveCommands(Set<String> selectedIds, String mgr);
    List<String> displayNames(List<AppSection> apps, Set<String> ids);
}

// service/implementation/CommandPlannerImp.java
@Service
public class CommandPlannerImp implements CommandPlanner {
    // deps: InstallCommandService, AppCatalogService (interfaces)
}
```
Unit-test all three (catalog id, unknown id, mixed; winget/choco strings).

## 7.8 Decision tree (moved from UI into `InstallController.start`, Phase 9 —
this phase just records the contract)

```
requiresInteractive(mgr) → prefix "sudo " when needed → st.runExternal = commands
else requiresSudo && !isRoot → st.popup = new SudoPopup(commands, new StringBuilder())
else → streaming execution (Phase 9)
```

## Definition of Done (Phase 7)

- [ ] Sections/Apps navigation + Space selection + `✓ n` badge + `[I]` markers (with stub installed data) — **pending human run in Windows Terminal, see below**
- [ ] `/` opens search popup; Enter shows (stub) results in main; Space selects; Esc returns to APPS — **pending human run**
- [ ] `l` shows installed view; `r` refresh; Esc restores; red styling on picked rows — **pending human run**
- [ ] `c` custom popup commits ids — **pending human run**
- [ ] `d` opens the confirm popup listing real command strings (e.g. `winget install --id Git.Git -e`); `y` reaches the stub, `n` cancels — **pending human run**
- [ ] Switching manager in panel `[2]` resets and refilters everything — **pending human run**
- [x] `CommandPlanner` unit tests green; controllers contain no command-string logic (SRP)

**Verification note:** `mvn compile`/`test` are green (44 tests incl. 5 new `CommandPlannerTest` cases).
`mise run dev` was launched from the agent session and reached `ToolkitApp.run()` (all beans,
including the new `CommandPlanner`/`CommandPlannerImp`, `PackageQueryService`/
`PackageQueryServiceImp`, and the catalog/search/installed views+controllers, resolve cleanly and
`TuiApp.onStart()` runs its OS-detect + PM-detect + controller-map setup), failing only at the same
`BackendException: Failed to get input console mode` recorded in Phases 5/6 — not a regression
(no `tmux` on this Windows machine; a background-launched process has no real console handle). The
interactive rows above need a human to run `mise run dev` in an actual Windows Terminal window.
See FEATURE-PARITY.md deviations for the Phase 7 implementation notes.
