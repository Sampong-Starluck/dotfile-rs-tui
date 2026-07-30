# Phase 8 — ScriptService + Shells panel / ShellInfo view

**Goal:** full port of `../src/service/script_service.rs` (filesystem logic)
plus the scripts feature in the lazygit layout (`[4]─Shells` side panel +
`SHELL_INFO` main view). Fully synchronous — no async work.

Note: base directories come from `config/AppProperties.dataDirName()`
(default `dotfile-rs` for interop with the Rust app) — inject it instead of
hardcoding the name.

## 8.1 `service/ScriptService.java` (port of `script_service.rs`)

Interface in `service/` listing every method below; implementation
`service/implementation/ScriptServiceImp` (`@Service`) injecting
`PathService`, `OsService`, `AppCatalogService` (interfaces), `ObjectMapper`,
and `AppProperties` (data dir name). Methods, 1:1 with the Rust:

### Constants / embedded scripts
The five profile scripts are classpath resources (copied in Phase 1):
`/scripts/bash/main_profile.sh`, `/scripts/zsh/main_profile.zsh`,
`/scripts/fish/main_profile.fish`, `/scripts/nu/main_profile.nu`,
`/scripts/posh/main_profile.ps1`. Loader:
`String scriptContent(String shellId)` → read resource as UTF-8, null for
unknown ids. `boolean hasScript(id)` → id ∈ {bash, zsh, fish, nushell, powershell}.

### Shell detection
```java
String shellBinary(String id)   // bash→bash, zsh→zsh, fish→fish, nushell→nu, powershell→pwsh, else null
boolean isShellDetected(String id) { return bin != null && pathService.isOnPath(bin); }
```

### Paths — port EXACTLY (Windows vs Unix branches)
```java
Path homeDir()          // Windows: %USERPROFILE%; Unix: $HOME; fallback tmp dir
Path scriptsBaseDir()   // Windows: %APPDATA%\dotfile-rs\scripts  (default home\AppData\Roaming)
                        // Unix: $XDG_DATA_HOME|~/.local/share / dotfile-rs/scripts
Path configBaseDir()    // Windows: %APPDATA%\dotfile-rs ; Unix: $XDG_CONFIG_HOME|~/.config /dotfile-rs
Path scriptTarget(id)   // base/bash/main_profile.sh, zsh/main_profile.zsh, fish/main_profile.fish,
                        // nu/main_profile.nu, posh/main_profile.ps1 ; null unknown
Path shellProfilePath(id)
    // bash:  mac|win → ~/.bash_profile ; linux → ~/.bashrc
    // zsh:   ~/.zshrc
    // fish:  win → %APPDATA%\fish\config.fish ; unix → $XDG_CONFIG_HOME|~/.config/fish/config.fish
    // nushell: win → %APPDATA%\nushell\config.nu ; unix → …/.config/nushell/config.nu
    // powershell: win → ~\Documents\PowerShell\Microsoft.PowerShell_profile.ps1
    //             unix → …/.config/powershell/Microsoft.PowerShell_profile.ps1
```
**Keep the `dotfile-rs` directory name** so existing deployments from the Rust
app remain visible/compatible.

### Source line
```java
String sourceHint(id)   // bash/zsh/fish/nushell: source "<target>"   powershell: . "<target>"
```

### Deploy / undeploy
```java
Path deployScript(id) throws ScriptException
    // mkdirs(parent), write scriptContent to scriptTarget, return path
void undeployScript(id)   // delete target if exists
```
Use a small checked `ScriptException(String msg)` mirroring the Rust
`Result<_, String>` — the UI shows the message.

### Profile add/remove (port :249–304 carefully)
```java
record ProfileResult(boolean changed, Path profile) {}
ProfileResult addSourceToProfile(id)
    // read profile (may not exist); if any trimmed line equals the source line → (false, path)
    // else append "\n# dotfile-rs\n<sourceLine>\n" (create dirs/file as needed) → (true, path)
ProfileResult removeSourceFromProfile(id)
    // port strip_source_block(): remove the exact 3-line block
    // [empty line, "# dotfile-rs", sourceLine] and any stray bare sourceLine;
    // preserve trailing newline iff the original had one
```

### Primary-shell config (port :308–354)
```java
Path configPath()                       // configBaseDir()/config.json
DotfileConfig readConfig()              // missing/corrupt → new DotfileConfig(null)
void writeConfig(DotfileConfig cfg)     // mkdirs + Jackson write
void setPrimaryShell(id) / clearPrimaryShell()
Optional<String> detectDefaultShell()
    // Windows: pwsh on PATH → "powershell" else empty
    // Unix: $SHELL basename → bash|zsh|fish|nu→nushell|pwsh/powershell→powershell
Optional<String> effectivePrimaryShell()  // config value, else detectDefaultShell()
Optional<String> chshCommand(id)        // Windows → empty; Unix → "chsh -s <which(bin)>"
```

### Status assembly (port :22)
```java
List<ShellStatus> loadShellStatuses()
    // for each readShellsJson() entry: detected = isShellDetected,
    // target = scriptTarget, deployed = target!=null && Files.exists(target)
```

## 8.2 Unit tests (use `@TempDir`; override base dirs via a settable
supplier or protected method so tests never touch the real `%APPDATA%`)

- deploy → file exists with resource content; undeploy → gone.
- addSourceToProfile on empty dir → creates profile containing
  `# dotfile-rs` + source line; second call → `changed == false` and file
  unchanged (idempotent).
- removeSourceFromProfile removes exactly the block; other lines intact;
  trailing-newline preserved; second call → `changed == false`.
- read/writeConfig round-trip; corrupt JSON → default (no throw).
- sourceHint("powershell") starts with `". \""`.

## 8.3 `ui/feature/scripts/` — ShellsPanel + ShellInfoView + ScriptsController

Behavior source `script_tab.rs`; structure per PLAN.md §5 (view classes render
only, controller mutates state + calls `ScriptService` — SRP).

### `ShellsPanel` (side panel `[4]─Shells`)
- Lazy load: `st.scriptShells == null` → `loadShellStatuses()` +
  `st.scriptPrimaryShell = effectivePrimaryShell().orElse(null)`.
- `Lists.selectable` rows: `primaryStar + statusIcon + label`:
  `★ ` yellow when primary; status icon — not detected `○` darkGray,
  deployed `✓` green, detected-only `◆` yellow; label = name (id when
  width < 18). Cursor styling comes from the component.

### `ShellInfoView` (`MainView.SHELL_INFO` — default main view while SHELLS focused)
Vertical split `[Fill(1), Length(clamp(h/3, 5, 12))]`: info on top, log below.
- **Info** (port `render_info`, :221): label/value rows Shell / Binary / Desc /
  Status / Platforms / Primary / Script dir / Profile — exact icons + colors:
  platform badges highlight the current OS (reversed cyan); primary line
  explicit → `"★  set as primary"` yellow bold, system default →
  `"◇  system default ($SHELL)"` cyan, else `"—"`; when detected, show the
  source line hint in yellow under a dim separator; `requires` list last.
- **Log**: the shared `Logs.colored` factory over `st.scriptLog`
  (`✓`→green, `✗`→red, `★`→yellow, default darkGray).

### `ScriptsController` (keys while SHELLS focused; port :27–48)
```
↓/j, ↑/k → st.shellCursor (clamped)
Enter    → deploy: deployScript + addSourceToProfile; log
           "  ✓ Script → <path>" / "  ✓ Added source line → <path>" /
           "  — Already in <path>" / "  ✗ … failed: <msg>"; st.scriptShells = null
d        → undeploy (guard "  — <name> is not deployed."), analogous logs
p        → setPrimary; log "  ★  <name> set as primary shell.";
           chshCommand present → log "  ▶  Running: <cmd>" + st.runExternal.add(cmd)
           (Windows: chsh empty → "  — chsh not available on this platform.")
c        → clearPrimary; re-detect default; log result
r        → st.scriptShells = null; log "  Refreshed shell status."
```
Hint bar (extend `Bindings`): `enter: deploy · d: undeploy · p: primary · c: clear · r: refresh`.

## Definition of Done (Phase 8)

- [ ] All ScriptService tests green; **deploy tests run only against @TempDir** (never the real APPDATA)
- [ ] Panel `[4]` lists the 5 shells; powershell shows `◆` if pwsh installed, bash likely `○`; main panel shows shell info while `[4]` focused
- [ ] Enter on PowerShell deploys: file appears at `%APPDATA%\dotfile-rs\scripts\posh\main_profile.ps1`, profile gains the dot-source line, log shows both ✓ lines, icon flips to `✓`
- [ ] `d` undeploys and cleans the profile line (verify file content manually once)
- [ ] `p` sets primary (★ appears; config.json written); `c` clears it
- [ ] Re-running `Enter` is idempotent ("— Already in …")
- [ ] View classes contain zero service calls; all mutations go through `ScriptsController`
