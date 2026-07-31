# dotfile-java-tui

Java port of [`dotfile-rs-tui`](../) — a lazygit-style TUI wrapper around your OS's package
manager(s) (winget/scoop/choco/apt/dnf/pacman/yay/paru/xbps/brew) plus a shell-profile script
manager (bash/zsh/fish/nushell/powershell). Built on Spring Boot 4.1 + TamboUI 0.4.0, running on
GraalVM JDK 25.

It shares its data directory with the Rust original — `%APPDATA%\dotfile-rs` on Windows
(`$XDG_CONFIG_HOME/dotfile-rs` or `~/.config/dotfile-rs` on Unix) — including the snake_case
`config.json`, so switching between the two builds is safe.

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
