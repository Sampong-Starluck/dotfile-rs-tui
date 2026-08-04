# Phase 5 — AppState, base abstractions, fluent components, lazygit shell

**Goal:** the application skeleton on the **Toolkit DSL (fluent API)** —
state object, `base/` contracts, reusable Element factories in
`ui/component/`, the lazygit frame in `ui/layout/LazygitLayout`, and
`TuiApp extends ToolkitApp` as the single entry point with working panel
focus. After this phase the app shows the full lazygit frame with
placeholder panel contents.

Behavior references: `../src/app.rs` (state fields), `../src/main.rs`
(loop semantics). UI structure: PLAN.md §5. API names: the Phase 1
`api-toolkit.txt` checklist (Rule #0).

## 5.1 `base/` — the MVC contracts (SOLID: small, stable interfaces)

TamboUI's own docs recommend this exact split — lean on it:

```java
package com.sampong.dotfile.base;

/** V: a pure function of state → fluent Element tree. No side effects. */
public interface FeatureView {
    Element render(AppState st);
}

/** C: handles a key for its feature. HANDLED stops propagation. */
public interface KeyController {
    EventResult handleKey(KeyEvent key, AppState st);
}

/** One modal at a time; a popup bundles its overlay view + key controller. */
public sealed interface Popup
        permits SearchInputPopup, CustomInputPopup, ConfirmActionPopup,
                SudoPopup, InstallLogPopup, HelpPopup {
    FeatureView view();
    KeyController controller();
}
```

Concrete popup records live in their feature packages and carry their own
small state (e.g. `SearchInputPopup(StringBuilder query)`). Create empty
stubs now so the sealed interface compiles.

## 5.2 `state/` — composed state management (PLAN.md §5a)

Plain mutable classes created inside `TuiApp` (never Spring beans). Field
sets port `../src/app.rs` minus the tab/focus bookkeeping the lazygit design
deletes (no `activeTab`, no `appFocus`, no `searchOrigin`, no
`pmPickerSelected`). **Rules:** views read, controllers/`frameTick` mutate
through named methods; invariants (cursor clamping, reset cascades) live in
the state classes; toolkit widget state objects (list/input state from
Phase 1 §1.3) are fields here, not created in views.

```java
/** root: navigation only + feature states */
public class AppState {
    public PanelId focused = PanelId.MANAGERS;   // mirrors toolkit focus (§5.5)
    public MainView mainView = MainView.COMMANDS;
    public Popup popup = null;                   // null = no modal
    public boolean running = true;

    public final PlatformState platform = new PlatformState();
    public final CatalogState catalog = new CatalogState();
    public final SearchState search = new SearchState();
    public final InstalledState installed = new InstalledState();
    public final InstallState install = new InstallState();
    public final ScriptsState scripts = new ScriptsState();

    /** cross-feature reset: switching the active manager invalidates
     *  everything derived from it (port of the Rust reset block) */
    public void activateManager(int idx) {
        if (!platform.activate(idx)) return;
        catalog.reset(); search.reset(); installed.reset();
    }
}

/** platform — NOTE: lazy per the 200 ms rule (PLAN.md §5b) */
public class PlatformState {
    public OperatingSystem os;                       // cheap: detected in constructor path
    public List<PackageManager> packageManagers = List.of();
    public boolean detecting = true;                 // true until async detect lands
    public CompletableFuture<List<PackageManager>> detectFuture = null;
    public int selectedPm = 0;                       // active manager (● marker)
    public int managersCursor = 0;
    public int commandScroll = 0;

    public String activeBinary() { … }               // "unknown" while empty
    public Optional<PackageManager> selectedManager() { … }
    public void moveCursor(int delta) { … }          // clamped here, not in controllers
    boolean activate(int idx) { … }                  // false when unchanged
}

public class CatalogState {
    public List<AppSection> apps = null;             // null = (re)load on next render
    public int sectionCursor = 0, appCursor = 0;
    public final Set<String> selectedIds = new LinkedHashSet<>();
    public void reset() { … }
}

public class SearchState {
    public List<SearchResult> results = new ArrayList<>();
    public int cursor = 0;
    public boolean loading = false;
    public String lastQuery = "";
    public CompletableFuture<List<SearchResult>> future = null;
    public void reset() { … }
}

public class InstalledState {
    public List<SearchResult> packages = new ArrayList<>();
    public final Set<String> names = new HashSet<>();
    public int cursor = 0;
    public boolean loading = false, autoLoaded = false, removeMode = false;
    public CompletableFuture<List<SearchResult>> future = null;
    public void reset() { … }
}

public class InstallState {                          // Phase 9
    public final List<String> log = new ArrayList<>();
    public ConcurrentLinkedQueue<String> logQueue = null;
    public LinkedBlockingQueue<String> stdinQueue = null;
    public final List<String> runExternal = new ArrayList<>();
    public boolean runExternalRemoving = false;
}

public class ScriptsState {
    public int shellCursor = 0;
    public final List<String> log = new ArrayList<>();
    public List<ShellStatus> shells = null;          // null = reload on next render
    public String primaryShell = null;
}
```

Later phases reference these as `st.platform.…`, `st.catalog.…`, etc. —
mentally rewrite any older `st.<field>` shorthand in phase files 6–10 to the
composed path.

## 5.3 `ui/component/` — reusable fluent Element factories (DRY)

Features may not hand-build panels, lists, inputs, logs, or hints — they
compose these static factories, which wrap the toolkit's fluent DSL:

- **`Panels.framed(int number, String title, boolean focusTracked)`** →
  `panel(…)` configured lazygit-style: title `"[n]─Title"`, `.rounded()`,
  `.id("panel-" + slug).focusable()`,
  `.borderColor(darkGray).focusedBorderColor(green)`. Overload without a
  number for main/popup panels.
- **`Lists.selectable(List<T> items, int cursor, boolean focused,
  Function<T,Element> row)`** → a `column()`/`list()` of row Elements with
  the cursor row **bold + reversed** (bold cyan when unfocused), scrolled so
  the cursor stays visible (use the toolkit list + selection state object if
  it scrolls natively — prefer built-in over hand-rolled).
- **`Inputs.line(StringBuilder value, String placeholder)`** → styled
  `text(value + "▌")` (dim bare `"▌"` when empty). Prefer the toolkit's
  text-input element + state if discovered in Phase 1; then `Inputs.line`
  just wraps it (single place to swap — DRY).
- **`Logs.colored(List<String> lines, int maxRows)`** → `column()` of the
  last N lines, colored by content: `✓`→green, `✗`/`[err]`→red, `▶`→cyan,
  `═`→yellow bold, `★`→yellow, default white/dim.
- **`Popups.overlay(Element content, int percentX, int percentY)`** →
  centered modal wrapper. Implementation: a custom `Element` whose
  `render(frame, area, context)` clears the centered sub-rect and renders
  `content` inside a framed panel — the toolkit-documented overlay pattern.
- **`Popups.confirm(String title, String warning, List<String> items,
  String yesLabel, String noLabel)`** → the lazygit confirm dialog (max 10
  items + "… and N more", `y`/`n` chip row).
- **`HintBar.of(List<Binding> bindings)`** → one dim line:
  `key: desc │ key: desc …`, keys cyan. (`record Binding(String key, String desc)`.)
- **`UiText`** — `truncate` (code-point, `…`), `padRight`, spinner frame
  chars only if the built-in `spinner()` can't be used somewhere.

## 5.4 `ui/layout/LazygitLayout.java` — the frame as one fluent composition

No Rect math — compose with the DSL's sizing:

```java
public static Element frame(Element status, Element managers, Element sections,
                            Element shells, Element main, Element hints,
                            int pmCount, int shellCount) {
    var side = column(
            status.height(3),
            managers.height(Math.min(pmCount, 6) + 2),
            sections,                                  // fill remaining
            shells.height(Math.min(shellCount, 7) + 2));
    return column(
            row(side.width(/* clamp(total/3, 24, 34) — use the DSL's
                              percentage/min/max sizing found in Phase 1 */),
                main),                                 // fill
            hints.height(1));
}
```

Exact sizing methods come from the §1.3 checklist; keep every sizing rule in
this one class.

## 5.5 `ui/TuiApp.java` — the ONE entry point (extends ToolkitApp)

`@Component @Slf4j @RequiredArgsConstructor`; constructor-injects service
interfaces + feature views/controllers (EnumMap registries — OCP). The
Phase 1 smoke-test runner bean is replaced by this bean. Creates `AppState`
in `run()`, launches the toolkit runner with the Panama backend.

```java
@Component
public class TuiApp extends ToolkitApp {
    private final AppState st;                       // created in run(), field for render()
    private final Map<PanelId, FeatureView> panelViews;
    private final Map<PanelId, KeyController> panelControllers;
    private final Map<MainView, FeatureView> mainViews;

    @Override
    protected Element render() {
        frameTick();                                  // §5.6
        Element root = LazygitLayout.frame(
                panelViews.get(STATUS).render(st),
                panelViews.get(MANAGERS).render(st),
                panelViews.get(SECTIONS).render(st),
                panelViews.get(SHELLS).render(st),
                mainViews.get(st.mainView).render(st),
                HintBar.of(Bindings.forState(st)),
                st.packageManagers.size(), shellCount(st));
        Element withKeys = root.onKeyEvent(e -> handleGlobal(e));
        return st.popup == null ? withKeys
                : stacked(withKeys, st.popup.view().render(st));   // overlay on top
    }
}
```

### Key routing (`handleGlobal`, root-level `.onKeyEvent`)

```
1. st.popup != null      → return st.popup.controller().handleKey(e, st)
2. '?'                   → st.popup = new HelpPopup(); HANDLED
3. 'q'                   → quit(); HANDLED
4. '1'..'4'              → programmatic focus of that panel id; HANDLED
5. otherwise             → panelControllers.get(st.focused).handleKey(e, st)
```

Focus itself: **let the toolkit own it.** Panels are focusable with ids
(§5.3); Tab/Shift-Tab/mouse work natively. Keep `st.focused` in sync via
the focus-change observation found in §1.3 (or by resolving the focused id
each render). MAIN gains focus programmatically on Enter-from-SECTIONS and
returns to SECTIONS on Esc. When focus lands on STATUS/MANAGERS →
`st.mainView = COMMANDS`; SECTIONS → APPS (unless INSTALLED/SEARCH_RESULTS
active); SHELLS → SHELL_INFO — one switch in the focus-sync method.

### `frameTick()` — per-render upkeep (§5.6)

Runs at the top of `render()`:
- **lazy platform detection (200 ms rule, PLAN.md §5b):** on the first tick,
  `st.platform.detectFuture = pmQuery.detectManagers()` (async wrapper over
  `PackageManagerService.detect()`), `.thenRun(this::requestRender)`; when
  done → set `packageManagers`, `detecting=false`. `AppState` construction
  itself does only the cheap OS detect — **no PATH probes on the startup
  path**; the Managers panel shows a spinner row while `detecting`
- drain `st.search.future` / `st.installed.future` when done (Phase 7)
- drain `st.install.logQueue` into `st.install.log` (Phase 9)
- lazy-load `st.catalog.apps` / `st.scripts.shells` when null (Phases 7–8)
  — each load wrapped in a debug timing log; if measured > 200 ms it must be
  converted to the async-future pattern above
- external-command trampoline: `!st.install.runExternal.isEmpty()` →
  `externalRunner.run(st, this)` (Phase 9; until then log + clear)

**Re-render trigger:** completion callbacks must wake the runner — attach
`.thenRun(this::requestRender)` (the refresh method found in §1.3) to every
future, and have the Phase 9 log bridge call it per event. While
`searchLoading || installedLoading`, the built-in `spinner()` element keeps
frames ticking, which also drives `frameTick()`.

## 5.6 `ui/Keys.java`

Static predicates over the real `KeyEvent`: `isUp` (↑ or 'k'), `isDown`
(↓ or 'j'), `isEnter`, `isEsc`, `isBackspace`, `isChar(k,c)`, `charOf(k)`
(printable or -1), `digitOf(k)` (1–4 or -1). Navigation predicates are for
list controllers only — text inputs consume `j`/`k` as characters via
`charOf`.

## Definition of Done (Phase 5)

- [x] App opens showing the full lazygit frame: 4 titled side panels `[1]…[4]`, main panel, hint bar — all via the fluent DSL (no immediate-mode `renderWidget` calls in app code except inside `Popups.overlay`) — verified by human in Windows Terminal
- [x] First frame appears instantly; Managers panel shows a spinner until async PM detection lands (200 ms rule) — startup timing lines visible in `debug.log` — verified by human
- [x] State lives only in `state/` classes; grep shows no mutable UI fields in views/components (feature views are stateless classes with a single `render(AppState)` method; `grep System\.out` clean)
- [x] Tab/Shift-Tab (framework) and `1`–`4` (programmatic) move focus; focused border turns green; `st.focused` stays in sync (log it) — verified by human
- [x] Clicking a panel focuses it (framework mouse focus) — verify, note result — verified by human
- [x] `q` quits cleanly; `?` opens a stub popup overlay and any key closes it — verified by human
- [x] Resize reflows the fluent layout without artifacts — verified by human
- [x] `base/`, `ui/component/`, `ui/layout/` exist as specified; feature packages stubbed
- [x] `mise run test` still green

**Verification note:** `mvn compile`/`test` are green and the Spring context wires
end-to-end (every bean, including `TuiApp`, resolves). The interactive rows above were
confirmed by a human running `mise run dev` in a real Windows Terminal window (the
agent session itself could not drive them — no `tmux` on this Windows machine, and a
background-launched process here gets no real console handle,
`BackendException: Failed to get input console mode`). See FEATURE-PARITY.md
deviations for the full note.
