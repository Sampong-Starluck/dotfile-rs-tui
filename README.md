# dotfile-java-tui

A lazygit-style TUI wrapper around your OS's package manager(s)
(winget/scoop/choco/apt/dnf/pacman/yay/paru/xbps/brew) plus a shell-profile script manager
(bash/zsh/fish/nushell/powershell). Built on Spring Boot 4.1 + TamboUI 0.4.0, running on GraalVM
JDK 25.

Originally built as a Java port of a Rust implementation of the same tool (that Rust source has
since been retired; its history is still in this repo's git log). Data directory:
`%APPDATA%\dotfile-rs` on Windows (`$XDG_CONFIG_HOME/dotfile-rs` or `~/.config/dotfile-rs` on
Unix), snake_case `config.json`.

## Toolchain

Managed by [mise](https://mise.jdx.dev/): GraalVM JDK 25 + Maven 3.9.

```powershell
mise install
```

## Commands

```powershell
mise run dev      # build + run the TUI
mise run test      # unit tests
mise run build     # fat jar -> target/dotfile-java-tui-0.1.0.jar
mise run native    # GraalVM native-image (Phase 11, best-effort on this machine)
```

Or, once built:

```powershell
java --enable-native-access=ALL-UNNAMED -jar target/dotfile-java-tui-0.1.0.jar
```

or the convenience launcher:

```powershell
.\dotfile.cmd
```

### Native image (Phase 11)

`mise run native` produces `target/dotfile-java-tui.exe` — run it directly, no
`java`/`--enable-native-access` flag needed. Requires the MSVC C++ build tools on `PATH`
(`cl.exe`); GraalVM native-image locates a Visual Studio install automatically if present. The
`native` Maven profile passes `-H:+UnlockExperimentalVMOptions -H:+SharedArenaSupport`, required
because TamboUI's Windows Panama backend closes a shared FFM `Arena`.

**Profile-guided optimization:** the baseline build already gets GraalVM's automatic ML-inferred
PGO. For a build tuned to real usage:

```powershell
mise run native-instrument   # -> target/dotfile-java-tui-sampling.exe
# run it in a real Windows Terminal, exercise the app (ideally the full walkthrough this
# README's keybinding table covers), then quit — it writes default.iprof to this directory
mise run native-pgo          # rebuilds target/dotfile-java-tui.exe using that profile
```

This uses `--pgo-sampling`, not the more common `--pgo-instrument` — the latter hits a GraalVM
25.0.3 native-image compiler crash on this project (a LIR register-allocator assertion inside the
FFM upcall stub TamboUI's Panama backend registers; see `FEATURE-PARITY.md`). `--pgo-sampling` is
GraalVM's own lower-overhead alternative and produces an equally usable `default.iprof`. A profile
collected from only the app's non-interactive startup path (no real console available in some
environments) already builds cleanly and shrinks the binary noticeably; a profile from the full
interactive walkthrough above would cover the actual TUI hot paths too and is the recommended way
to (re)generate `default.iprof` before relying on `native-pgo` for a release build.

## Keybindings

Always available: `1-4` jump to a side panel · `tab`/`shift-tab` cycle focus · `?` help · `q` quit.

| Context | Bindings |
|---|---|
| Managers / Status panel | `enter` use manager · `j`/`k` move · `pgup`/`pgdn` scroll commands |
| Sections panel | `enter` open · `/` search · `l` installed · `c` custom package id · `d` install |
| Main · Apps | `space` select · `d` install · `/` search · `l` installed · `esc` back |
| Main · Search results | `space` select · `d` install · `/` edit query · `esc` back |
| Main · Installed | `space` select · `d` remove · `r` refresh · `esc` back |
| Shells panel | `enter` deploy · `d` undeploy · `p` set primary · `c` clear primary · `r` refresh |
| Help popup | `esc`/`?` close |
| Search / custom-id popup | `enter` confirm · `esc` cancel |
| Confirm popup | `y` yes · `n` no |
| Sudo popup | `enter` run · `esc` cancel |
| Install log popup | type + `enter` respond to a prompt · `esc` close |

(Generated from `ui/Bindings.java`, the single source shared by the in-app hint bar and the `?`
help popup — if a binding ever changes, update it there and this table together.)

## Project status

See `CLAUDE.md` and `FEATURE-PARITY.md` for the migration-plan status and the full Rust → Java
behavior mapping.
