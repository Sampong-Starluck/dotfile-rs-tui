# UX/UI Improvement Plan — dotfile-rs-tui

> Goal: close the UX gap with **lazygit** in the two areas where it leads us —
> **discoverability** (the user can find every action) and **trust** (the user
> always knows *what* an action will do before confirming). Keep our existing
> visual language (rounded borders, cyan = focused, colored key-chips), which is
> already clean and in some places better than lazygit.
>
> Each task below is self-contained. It states the *current behavior*, the
> *target behavior*, the *exact files/lines* to touch, *step-by-step* changes,
> and *acceptance criteria* so both a human and an AI agent can execute and
> verify it without re-deriving context.

---

## Conventions used in this document

- **File refs** use `path:line` from the state of the repo when this plan was
  written. Line numbers drift after edits — search for the quoted code instead
  of trusting the number blindly.
- **Focus model recap** (`src/enumerate/app_focus.rs`): the Application tab has
  8 focus states — `Section`, `Apps`, `CustomInput`, `Search`, `Installed`,
  `Installing`, `SudoConfirm`, `PmPicker`. Many features below branch on these.
- **Status bar** lives in `src/ui/mod.rs::render_status` (the colored key-chips).
- **Global keys** (Tab/BackTab/q) are handled in `src/main.rs::run`, lines ~108–134.
- Verify each task with `cargo build` then `cargo run`, exercising the tab/focus
  state involved. There is no automated UI test harness, so verification is manual.

---

## Priority 1 — High impact, do first

### Task 1.1 — Add a global `?` help overlay

**Why:** lazygit's biggest discoverability win is the `?` keybinding popup. Our
status bar only shows 3–5 hints and entirely omits some bindings (e.g. `p` =
PM picker). New users cannot discover the full command set.

**Current state:**
- No help overlay exists.
- `src/app.rs` `App` struct has no "show help" flag.
- Global key handling in `src/main.rs:108` handles only Tab/BackTab/q.

**Target state:**
- Pressing `?` (when *not* in a text-input focus) toggles a centered modal
  listing every keybinding, grouped by the current tab / focus.
- Pressing `?` again, `Esc`, or `q` closes it.
- While the help overlay is open, all other keys are swallowed (no navigation
  underneath).

**Steps:**
1. `src/app.rs`:
   - Add field `pub show_help: bool,` to `struct App`.
   - Initialize `show_help: false,` in `App::new()`.
   - (Optional) extend `is_text_input_focus` usage — help must NOT open while
     typing. We gate on `!app.is_text_input_focus()` at the call site instead.
2. `src/main.rs` in the global key `match key.code` (around line 109):
   - Add an arm BEFORE the tab delegation:
     ```rust
     KeyCode::Char('?') if !app.is_text_input_focus() => {
         app.show_help = !app.show_help;
     }
     ```
   - Add an early arm so that *while help is open* `Esc`/`q`/`?` close it and
     everything else is ignored:
     ```rust
     _ if app.show_help => {
         if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')) {
             app.show_help = false;
         }
         // swallow all other keys while help is visible
     }
     ```
     Place this arm so it runs before the per-tab delegation but after Tab
     handling is fine either way — simplest is: at the very top of the inner
     `match`, `if app.show_help { ...toggle/ignore...; }` guard.
3. New rendering: in `src/ui/mod.rs::render`, after `render_status(...)`, add:
   ```rust
   if app.show_help {
       help::render_help(frame, app);
   }
   ```
   Create `src/ui/help.rs` (and `mod help;` in `src/ui/mod.rs`) with a
   `render_help(frame, app)` that:
   - Builds a `centered_rect` (reuse the pattern from `app_tab.rs:605`; consider
     extracting `centered_rect` into a shared `ui` helper — see Task 3.4).
   - Renders a `Clear` then a bordered `List`/`Paragraph` of key→description rows.
   - Section the content by context: a "Global" group (Tab, Shift+Tab, q, ?) plus
     a group for `app.active_tab`, and for the Application tab, a sub-group per
     `app.app_focus`. Reuse the same pairs already defined in
     `render_status` so there is a single source of truth (see Task 1.4).

**Acceptance criteria:**
- `?` opens the overlay from Home, Apps (Section/Apps focus), and Shell.
- `?` does NOT open while typing in Search / CustomInput / SudoConfirm / Installing.
- Overlay lists `p` (PM picker), `i`, `l`, `d`, Space, Enter, Esc — i.e. bindings
  currently missing from the status bar.
- `Esc`/`q`/`?` close it; underlying view is unchanged after closing.
- Add `("?", "help")` to the status-bar hint sets (Task 1.4 covers this).

---

### Task 1.2 — Resolve the placeholder Shell tab

**Why:** lazygit ships no "coming soon" stubs. Our Shell tab
(`src/ui/features/shell_tab.rs`) renders a literal "Shell scripts coming soon."
panel, which makes the whole app feel unfinished and lowers trust in the real
features.

**Current state:** `shell_tab.rs:32-52` renders placeholder text; `handle_key`
is a no-op. `TabModel::Shell` is one of three tabs in `src/ui/tabs.rs`.

**Target state — pick ONE (decision required from maintainer):**
- **Option A (recommended, lowest risk):** Remove the Shell tab entirely until
  it has real functionality.
  - In `src/models/tab_model.rs`, remove `Shell` from the enum and `ALL`.
  - Remove the `TabModel::Shell` arms in `src/ui/tabs.rs`, `src/ui/mod.rs`
    (`render`, `render_status`), and `src/main.rs` (`run` delegation).
  - Delete `src/ui/features/shell_tab.rs` and its `mod`/`pub use` wiring in
    `src/ui/features/mod.rs`.
- **Option B:** Keep the tab but turn it into an honest, non-stub state — e.g.
  a panel that lists detected dotfile scripts (even if read-only for now) with a
  clear "read-only preview" label rather than "coming soon."

**Acceptance criteria:**
- App builds and runs with no "coming soon" text reachable by the user.
- If Option A: Tab cycling moves Home ↔ Apps only; no dangling `Shell` references
  (`grep -ri shell src/` returns only intentional matches).
- If Option B: the Shell tab shows real, current information with no future-tense
  placeholder copy.

---

### Task 1.3 — Show package names in install/remove confirmation

**Why:** lazygit always shows *what* will be affected before a destructive or
significant action. Our sudo confirmation (`app_tab.rs:1828 render_sudo_confirmation`)
shows only a count: `"{count} package(s) selected."` The user cannot verify the
selection before typing their password.

**Current state:**
- `render_sudo_confirmation` shows manager name + count + y/n.
- The selected ids live in `app.app_selected_ids: HashSet<String>`.
- Note: many managers are "interactive" and bypass this modal entirely (they run
  in the real terminal via `app.run_external`, see `start_install`
  `app_tab.rs:1636` and `main.rs:66`). This task covers BOTH paths.

**Target state:**
- The sudo confirmation modal lists the package names (resolved display names
  where possible, falling back to the id) that will be installed/removed, capped
  to e.g. the first 10 with a "+N more" line if longer.
- The pre-external-run banner printed in `main.rs::run_in_terminal` already echoes
  each command (`▶ {cmd}`) — that is acceptable for the external path, but add a
  one-line summary "Installing N package(s): a, b, c …" before the loop for
  symmetry.

**Steps:**
1. Add a helper in `src/ui/features/app_tab.rs`:
   ```rust
   /// Human-readable names for the currently selected ids, resolved against the
   /// loaded apps catalog, falling back to the raw id (search/custom entries).
   fn selected_display_names(app: &App) -> Vec<String> { /* iterate app.app_selected_ids,
       look up in app.apps sections by id → entry.name, else use id */ }
   ```
2. In `render_sudo_confirmation`, replace the single count line with the count
   plus a wrapped, bulleted/comma list from `selected_display_names`, truncated
   with `truncate(..)` (already in this file) and a "+N more" overflow line.
3. In `main.rs::run_in_terminal`, before the `for cmd in commands` loop, print a
   summary line listing the package count (commands are already echoed per-line).

**Acceptance criteria:**
- Selecting 3 apps and pressing `d` on a sudo-required, non-interactive manager
  shows their names in the confirmation modal.
- A long selection (>10) shows the first N and a "+N more" line, without
  overflowing the modal.
- Names resolve to friendly `entry.name` for catalog apps and fall back to the
  raw id for search/custom ids.

---

### Task 1.4 — Single source of truth for keybinding hints (enabler for 1.1)

**Why:** the status bar (`src/ui/mod.rs`) and the new help overlay (Task 1.1)
must not drift apart. Today the hint pairs are inlined in `render_status`.

**Current state:** `render_status` builds `Vec<Span>` directly from inline
`hints(&[(key, desc), ...])` calls per `(tab, focus)`.

**Target state:** a single function returns the `(key, desc)` pairs for a given
`(TabModel, AppFocus)`; both the status bar and the help overlay consume it.

**Steps:**
1. Add `fn bindings_for(tab: TabModel, focus: AppFocus) -> Vec<(&'static str, &'static str)>`
   (e.g. in `src/ui/mod.rs` or a new `src/ui/keymap.rs`).
2. Move every `(key, desc)` set currently inside `render_status` into `bindings_for`.
3. `render_status` calls `bindings_for(...)` then renders chips.
4. Add the missing `("p", "pm")` to the Section/Apps sets and `("?", "help")` to
   all non-text-input sets.
5. The help overlay (Task 1.1) renders the same `bindings_for` output, optionally
   for *all* focus states of the active tab.

**Acceptance criteria:**
- Changing a hint in `bindings_for` updates both the status bar and the help overlay.
- Status bar now shows `p pm` in Section/Apps and `? help` everywhere it's valid.

---

## Priority 2 — Medium impact / polish

### Task 2.1 — Stop using red as the *theme* of the Installed panel

**Why:** red universally signals error/danger. The installed view currently
themes its border red (`app_tab.rs:492 .border_style(...fg(Color::Red))`) even
when nothing is being removed. lazygit reserves red for actual deletions/errors.

**Current state:**
- `render_installed_panel` uses red border always.
- Row highlight uses red bg only when an item is *picked for removal* — that part
  is correct and should stay.

**Target state:**
- Installed panel border uses a neutral focused accent (e.g. `Color::Magenta` or
  the standard cyan focused color from `panel(title, true)`).
- Keep the red row-highlight for items actively selected for removal — red here
  is meaningful ("this will be removed").

**Steps:**
1. In `render_installed_panel` (`app_tab.rs:413`), change the loading and main
   `.border_style(Style::default().fg(Color::Red))` to a neutral focused accent
   (or just drop the override so `panel(&title, true)` cyan is used).
2. Leave the per-row red bg in the `match (is_cursor, is_picked)` block alone.

**Acceptance criteria:**
- Opening the installed view (`l`) shows a non-red panel border.
- Selecting a package for removal still highlights that row in red.

---

### Task 2.2 — Move keybindings out of panel titles into the status bar

**Why:** the installed panel title hardcodes hints
(`app_tab.rs:420`: `"… [Space] select [d] remove [r] refresh"`). The status bar
already does this better and consistently (colored chips). Duplicated, differently
styled hints look cluttered.

**Current state:** `render_installed_panel` title embeds the hint string.

**Target state:** title is just `"Installed [{mgr}] — {n} package(s)"`; the
`Installed` focus status-bar set (already present in `render_status`) carries the
hints.

**Steps:**
1. In `render_installed_panel`, shorten the non-loading `title` to drop the
   `[Space]/[d]/[r]` text.
2. Confirm `render_status`' `AppFocus::Installed` arm already lists navigate /
   select / remove / refresh / back (it does) — keep it as the single place.

**Acceptance criteria:**
- Installed panel title shows only manager + count.
- The same hints still appear in the bottom status bar.

---

### Task 2.3 — Add `p` (PM picker) and `?` hints to all Application focus states

**Why:** `p` opens the package-manager picker (`handle_section_keys`/
`handle_apps_keys` arms for `'p'`) but is absent from the status bar, so it is
effectively hidden. (Largely handled by Task 1.4; this is the verification slice.)

**Steps:** ensure `bindings_for` (Task 1.4) includes `("p", "pm")` for `Section`
and `Apps`, and `("?", "help")` for every non-text-input focus.

**Acceptance criteria:** `p` and `?` are visible in the status bar in Section and
Apps focus; pressing `p` opens the picker.

---

## Priority 3 — Low impact / nice-to-have

### Task 3.1 — Make "press Enter to search" clearer when a query is typed

**Why:** in Search focus, after typing a query the user may not realize Enter
runs the search (typing only updates the buffer — see `handle_search_keys`
`app_tab.rs:777`). The input panel (`render_custom_input`, title `⌕ Search`) gives
no run cue.

**Target state:** when `app.search_query` is non-empty and not loading, the search
input border title or a trailing hint shows `↵ to search`.

**Steps:** in `render_custom_input` (`app_tab.rs:513`), in the `in_search` branch,
append a dim `↵ search` hint to the title or content when the buffer is non-empty.

**Acceptance criteria:** typing in search shows a visible "Enter to search" cue
that disappears while loading.

---

### Task 3.2 — De-duplicate the modal render calls in `app_render`

**Why:** code-health (not user-facing), but reduces future UX bugs. In
`app_render` (`app_tab.rs:110-127`) the sudo-confirm and install modals are each
rendered **twice** (once in the first `if/else if`, again in the second `if/else if`).

**Steps:** remove the redundant second block so each modal renders once.

**Acceptance criteria:** sudo and install modals still appear; no double-draw;
`cargo build` clean.

---

### Task 3.3 — Empty-state guidance parity

**Why:** lazygit gives helpful empty-state hints. We already do in several places
(e.g. "Type a query below and press Enter"). Audit the rest:
- Home with no package managers (`home_tab.rs:47`) — good.
- Apps with no section (`app_tab.rs:204`) — good.
- Installed empty (`app_tab.rs:438`) — good.

**Target:** confirm each empty state names the key that resolves it. Mostly done;
this is a verification/polish pass, not new work.

---

### Task 3.4 — Extract shared `centered_rect` helper (enabler / cleanup)

**Why:** `centered_rect` is defined in `app_tab.rs:605` and will be needed by the
help overlay (Task 1.1). Avoid copy-paste.

**Steps:** move `centered_rect` to a shared `ui` module (e.g. `src/ui/mod.rs` or
`src/ui/util.rs`), make it `pub(crate)`, and update callers.

**Acceptance criteria:** one definition; both `app_tab.rs` modals and the help
overlay use it; build clean.

---

## Suggested execution order

1. **1.4** (keymap single-source) — small, unblocks 1.1 and 2.3.
2. **3.4** (shared `centered_rect`) — small, unblocks 1.1.
3. **1.1** (help overlay) — the headline discoverability win.
4. **1.3** (names on confirm) — the headline trust win.
5. **1.2** (resolve Shell stub) — decision needed: remove vs. make real.
6. **2.1, 2.2, 2.3** (installed-panel color, title hints, p/? hints) — quick polish.
7. **3.1, 3.2, 3.3** (search cue, de-dupe modals, empty-state audit) — cleanup.

## Definition of done (whole plan)
- `?` reveals every binding; status bar and overlay never disagree (Task 1.4).
- No "coming soon" copy reachable by the user.
- Every install/remove confirmation names the affected packages.
- Red appears only on active removal, never as ambient theme.
- `cargo build` is warning-free for touched files; each task manually verified by
  running the app and exercising the relevant tab/focus.