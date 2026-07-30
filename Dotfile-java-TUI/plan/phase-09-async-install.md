# Phase 9 — Spring async workers: search, installed list, install/remove, external execution

**Goal:** replace all stubs with real process execution using **Spring's
async stack**: `@Async` methods on virtual threads (`AsyncConfig` +
`spring.threads.virtual.enabled=true`), `CompletableFuture` for one-shot
jobs, `ApplicationEventPublisher` for the streaming install log. The UI
thread never blocks.

Behavior sources: `app_tab.rs` (`run_search` :794, `run_list_installed`
:1464, install/remove executors, `drain_stdout_to_log` :825) and
`main.rs::run_in_terminal` :164.

## 9.1 `service/PackageQueryService` — one-shot @Async jobs

Interface (`search`, `listInstalled`, `detectManagers` — the Phase 5 lazy
detection wrapper) + `implementation/PackageQueryServiceImp`. `@Async` goes
on the Imp methods:

```java
@Slf4j
@Service
@RequiredArgsConstructor
public class PackageQueryServiceImp implements PackageQueryService {
    private final SearchService searchService;   // command tables, Phase 3

    @Async
    public CompletableFuture<List<SearchResult>> search(String mgr, String query) {
        try {
            var cmd = searchService.searchCommand(mgr, query);
            byte[] out = runAndCapture(cmd);
            return CompletableFuture.completedFuture(
                OutputParsers.parseSearchOutput(mgr,
                    OutputParsers.decodeSearchOutput(mgr, out)));
        } catch (Exception e) {
            log.error("search via {} failed", mgr, e);
            return CompletableFuture.completedFuture(List.of());
        }
    }

    @Async
    public CompletableFuture<List<SearchResult>> listInstalled(String mgr) {
        // same shape with listCommand(mgr); decode winget with the UTF-16-aware
        // decoder, everything else UTF-8; parse with parseListOutput
    }

    private byte[] runAndCapture(SearchService.Cmd cmd) throws IOException, InterruptedException {
        var pb = new ProcessBuilder(prepend(cmd.binary(), cmd.args()));
        pb.redirectError(ProcessBuilder.Redirect.DISCARD);
        Process p = pb.start();
        byte[] out = p.getInputStream().readAllBytes();
        p.waitFor();
        return out;
    }
}
```

Controllers already poll these futures per frame (Phase 7 §7.1) — delete the
fake-data stubs, wire the real service, done. **Never** call `join()`/`get()`
on a pending future from the UI thread.

## 9.2 Streaming install/remove — events + `service/InstallExecutionService`

`event/InstallLogEvent.java`:

```java
public record InstallLogEvent(String line) {}
```

`ui/feature/install/InstallLogBridge.java` — the single listener bridging
Spring events to the UI (SRP: it only appends):

```java
@Component
public class InstallLogBridge {
    private volatile ConcurrentLinkedQueue<String> sink;   // set by InstallController per run
    public void attach(ConcurrentLinkedQueue<String> q) { this.sink = q; }

    @EventListener
    public void on(InstallLogEvent e) {
        var q = sink;
        if (q != null) q.add(e.line());
        tuiApp.requestRender();     // wake the toolkit render loop (§1.3 name)
    }
}
```
(Same for the query futures: `future.thenRun(tuiApp::requestRender)` — the
Toolkit only re-renders when asked; PLAN.md §6.)

`service/InstallExecutionService` (interface: `runStreaming(...)`) +
`implementation/InstallExecutionServiceImp`:

```java
@Slf4j
@Service
@RequiredArgsConstructor
public class InstallExecutionServiceImp implements InstallExecutionService {
    private final ApplicationEventPublisher events;
    private final SystemService system;

    @Async
    public void runStreaming(List<String> commands, String sudoPassword,
                             BlockingQueue<String> stdinQueue) {
        for (String cmd : commands) {
            events.publishEvent(new InstallLogEvent("▶ Running: " + cmd));
            List<String> parts = List.of(cmd.trim().split("\\s+"));
            if (parts.isEmpty() || parts.getFirst().isEmpty()) continue;
            try {
                Process child = new ProcessBuilder(parts).start();

                var stdin = new PrintWriter(new OutputStreamWriter(
                        child.getOutputStream(), StandardCharsets.UTF_8), true);
                if (sudoPassword != null) stdin.println(sudoPassword);
                Thread relay = Thread.ofVirtual().start(() -> {
                    try { while (true) stdin.println(stdinQueue.take()); }
                    catch (InterruptedException stop) { }
                });

                Thread errPump = Thread.ofVirtual().start(() -> pumpStderr(child));
                drainStdout(child);                    // §9.3, blocks to EOF
                errPump.join();
                int code = child.waitFor();
                relay.interrupt();
                events.publishEvent(new InstallLogEvent(code == 0
                        ? "✓ Done: " + parts.getFirst()
                        : "✗ Failed (exit " + code + ")"));
            } catch (Exception e) {
                events.publishEvent(new InstallLogEvent(
                        "✗ Could not run " + parts.getFirst() + ": " + e.getMessage()));
            }
        }
        events.publishEvent(new InstallLogEvent("═══ All done ═══"));
    }

    private void pumpStderr(Process child) {
        // BufferedReader over stderr; per line: sanitizeLine → skip isNoiseLine,
        // skip lines containing "[sudo]" or "password for" → publish "[err] <line>"
    }
    private void drainStdout(Process child) { /* §9.3 */ }
}
```

## 9.3 `drainStdout` (port of `drain_stdout_to_log`)

Read stdout fully to bytes (winget's `\r` animation makes line-streaming
meaningless), then split on `\r` and `\n`, `stripAnsi` each segment, drop
`isNoiseLine` segments, publish the rest as `InstallLogEvent`s.

## 9.4 `ui/feature/install/` — InstallController, InstallLogPopup, SudoPopup

### `InstallController.start(st, kind, commands)` — the decision tree (§7.8)

```java
String mgr = st.activePackageManager();
if (system.requiresInteractive(mgr)) {
    List<String> cmds = (system.requiresSudo(mgr) && !system.isRoot())
            ? commands.stream().map(c -> "sudo " + c).toList() : commands;
    st.runExternal.addAll(cmds);
    st.runExternalRemoving = (kind == REMOVE);
    return;                                        // trampoline handles it (§9.5)
}
if (system.requiresSudo(mgr) && !system.isRoot()) {
    st.popup = new SudoPopup(kind, commands, new StringBuilder());
    return;
}
begin(st, commands, null);
```

`begin(st, commands, sudoPassword)`:

```java
st.installLog.clear();
st.installLog.add(kind == REMOVE ? "Starting removal…" : "Starting installation…");
if (system.requiresInteractive(mgr)) st.installLog.add("Type y/n and press Enter to respond.");
st.installLogQueue = new ConcurrentLinkedQueue<>();
st.installStdinQueue = new LinkedBlockingQueue<>();
logBridge.attach(st.installLogQueue);
st.popup = new InstallLogPopup();
executionService.runStreaming(commands, sudoPassword, st.installStdinQueue);
```

### `InstallLogPopup`
`Popups.overlay(…, 72, 72)`, a `column(log, input.height(3))`. Top:
`Logs.colored` over `st.installLog` (drained from `installLogQueue` in
`frameTick`), autoscrolled, yellow border, title `"⬇ Installing"`.
Bottom: `Inputs.line` titled `"respond — y/n · enter: send · esc: close"`.
Controller: chars/backspace edit; Enter → append `"> <input>"` to log +
`st.installStdinQueue.offer(input)`; Esc → close popup, clear queues/input/
`selectedIds`/log; if `st.removeMode` → refresh installed view, else focus
SECTIONS. (Worker runs to completion detached — same as the Rust.)

### `SudoPopup(kind, commands, password)`
`Popups.confirm`-based: `"⚠ <mgr> requires sudo"` yellow, display names of the
selection (via `CommandPlanner.displayNames`), masked password row
(`"•".repeat(len) + "▌"`). Enter (non-empty) → close; commands each prefixed
`"sudo -S "`; `begin(st, prefixed, password)`. Esc → cancel fully.

## 9.5 `ui/ExternalRunner.java` — suspend-TUI execution (port `run_in_terminal`)

Invoked by the `TuiApp` trampoline when `st.runExternal` is non-empty:

```java
public void run(AppState st, /* runner/terminal handle */) {
    List<String> commands = List.copyOf(st.runExternal);
    st.runExternal.clear();
    boolean wasRemoving = st.runExternalRemoving;
    st.runExternalRemoving = false;

    // 1. Suspend via the runner's pause/suspend API (Phase 1 §1.3).
    //    Required semantics: leave alt screen, raw mode off, cursor visible.
    //    If 0.4.0 has no such API, closing and recreating the runner is the
    //    correct implementation — do that cleanly, no half-measures.

    for (String cmd : commands) {
        System.out.println("\n[36m▶  " + cmd + "[0m");
        try {
            int code = new ProcessBuilder(List.of(cmd.trim().split("\\s+")))
                    .inheritIO().start().waitFor();
            System.out.println(code == 0 ? "[32m✓  Done[0m"
                    : "[31m✗  Exited with status " + code + "[0m");
        } catch (Exception e) {
            System.out.println("[31m✗  Failed: " + e.getMessage() + "[0m");
        }
    }
    System.out.print("\n[33m  Press Enter to return…[0m  ");
    System.out.flush();
    new BufferedReader(new InputStreamReader(System.in)).readLine();

    // 2. Resume the TUI.
    // 3. State reset (port main.rs:71-80):
    st.selectedIds.clear();
    st.removeMode = false;
    if (wasRemoving) { st.installedPackages.clear(); st.installedSet.clear(); }
    st.focused = PanelId.SECTIONS;
}
```
These are the only permitted `System.out` calls in the app (TUI is suspended).

On Windows winget/scoop never take this path, but **choco does** — test with
choco if installed; otherwise temporarily classify scoop as interactive to
verify suspend/resume, then revert.

## Definition of Done (Phase 9)

- [ ] Real `winget search git` results appear; UI responsive + spinner ticks while loading
- [ ] Installed view shows real `winget list`; `[I]` markers correct in APPS view
- [ ] Confirm → install of a small winget package streams log via `InstallLogEvent`s and ends `═══ All done ═══`
- [ ] Remove flow works from the installed view
- [ ] Esc mid-install closes the popup without freezing
- [ ] External path (choco or forced test): suspend → live command with prompts → Enter → clean restore + state reset
- [ ] All async goes through `@Async` beans; zero raw `new Thread`/`Thread.ofVirtual` outside `InstallExecutionService`'s relay/pump threads
- [ ] Phase-7 stubs deleted
