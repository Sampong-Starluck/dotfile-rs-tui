# Phase 10 — Help popup, key hints, packaging, acceptance

**Goal:** the remaining chrome and the shippable fat jar.

## 10.1 `ui/Bindings.java`

```java
public record Binding(String key, String desc) {}
public static List<Binding> forState(AppState st) { … }
```
Context table (lazygit-terse wording). Global suffix always appended:
`1-4: jump · tab: cycle · ?: help · q: quit`.

| Context | Bindings |
|---|---|
| popup = HelpPopup | `esc/?: close` |
| popup = Search/Custom input | `enter: confirm · esc: cancel` |
| popup = Confirm | `y: yes · n: no` |
| popup = Sudo | `enter: run · esc: cancel` |
| popup = InstallLog | `type+enter: respond · esc: close` |
| MANAGERS / STATUS | `enter: use manager · j/k: move · pgup/pgdn: scroll` |
| SECTIONS | `enter: open · /: search · l: installed · c: custom · d: install` |
| MAIN·APPS | `space: select · d: install · /: search · l: installed · esc: back` |
| MAIN·SEARCH_RESULTS | `space: select · d: install · /: edit query · esc: back` |
| MAIN·INSTALLED | `space: select · d: remove · r: refresh · esc: back` |
| SHELLS | `enter: deploy · d: undeploy · p: primary · c: clear · r: refresh` |

`HintBar.of` (Phase 5 factory) renders this — nothing else may print hints
(DRY).

## 10.2 `ui/feature/help/HelpPopup`

`Popups.overlay(…, 60, 70)`, title `"? Keybindings"`. Sections: Global, then every
context row from the table above, grouped with yellow-bold dim-line headers
(`── Section ──…`), key chips 16 wide reversed-cyan + white description.
Content is **generated from `Bindings`** — the popup owns zero key text
(single source of truth). Opens with `?`, closes on `?`/`q`/Esc, swallows
everything else.

## 10.3 Mouse

Click-to-focus on panels comes from the toolkit's focus system (verified in
Phase 5) — nothing to build. Optional extras, only if the toolkit exposes
them first-class (Phase 1 §1.3 notes): row-click to move a list cursor and
wheel-scroll on the hovered list / commands table. If not exposed in 0.4.0:
skip, one line in FEATURE-PARITY.md, ensure stray mouse escape sequences
don't crash the app.

## 10.4 Packaging

- `mise run build` → `target/dotfile-java-tui-0.1.0.jar`
- Verify from a fresh PowerShell (with mise env active):
  `java --enable-native-access=ALL-UNNAMED -jar target/dotfile-java-tui-0.1.0.jar`
- `README.md` at project root: what it is, toolchain (`mise install`), run/
  build/test commands, keybinding table (paste from `Bindings`), note that
  this is the Java port of `dotfile-rs-tui` and shares its
  `%APPDATA%\dotfile-rs` data dir.
- Convenience launcher `dotfile.cmd`:
  `@java --enable-native-access=ALL-UNNAMED -jar "%~dp0target\dotfile-java-tui-0.1.0.jar" %*`

## 10.5 Final acceptance run (Windows Terminal, top to bottom)

1. `mise run dev` → first frame instantly; panel `[2]` spinner resolves to
   real managers; Status shows `Windows · winget`. Check `debug.log` startup
   timings: no synchronous startup step > 200 ms (PLAN.md §5b).
2. Panel `[2]`: cursor previews command tables; Enter activates a manager.
3. Panel `[3]` → Enter into APPS; Space-select 2 apps → `✓ 2` in title;
   `[I]` markers present after installed auto-load.
4. `/` → search `terminal` → real results; Space select; Esc back.
5. `l` → installed list; `r` refresh; Esc back.
6. `c` → add `Microsoft.VisualStudioCode`; `d` → confirm popup lists exact
   winget commands; `y` → streaming install log → `═══ All done ═══`; Esc.
7. Remove flow: `l`, select something harmless, `d`, confirm, verify.
8. Panel `[4]`: deploy/undeploy powershell round-trip; `p`/`c` primary.
9. `?` help on several contexts; hint bar correct in every state.
10. Resize repeatedly (incl. < 80 cols) — layout holds, no artifacts.
11. `q` → clean exit, prompt intact, `debug.log` populated, console clean.

## Definition of Done (Phase 10)

- [ ] Hint bar context-correct in all states of the table above
- [ ] Help popup generated from `Bindings` (change a binding → both update)
- [ ] Mouse works or is documented unsupported
- [ ] Fat jar runs standalone; README + launcher script written
- [ ] Acceptance run §10.5 passes end-to-end
- [ ] 200 ms rule audit: `debug.log` timing lines reviewed; every slow step is lazy with a spinner placeholder
- [ ] Convention audit: all services interface+`Imp`; no `…Imp` referenced outside `service/implementation/`; no state outside `state/`; Lombok audit — no hand-written loggers/DI constructors, no `@Data`/`@Setter`/`@SneakyThrows` anywhere (PLAN.md §4a)
- [ ] FEATURE-PARITY.md fully ticked or annotated
