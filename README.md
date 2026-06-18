# ![Application Icon for Edit](./assets/edit.svg) Edit

A simple editor for simple needs.

This editor pays homage to the classic [MS-DOS Editor](https://en.wikipedia.org/wiki/MS-DOS_Editor), but with a modern interface and input controls similar to VS Code. The goal is to provide an accessible editor that even users largely unfamiliar with terminals can easily use.

> **About this fork.** This is an independent custom fork of
> [microsoft/edit](https://github.com/microsoft/edit), maintained by
> [mikuta0407](https://github.com/mikuta0407). It carries personal customizations and is **not**
> intended to be contributed back upstream. The original work is © Microsoft Corporation and
> licensed under the MIT License — see [`LICENSE`](./LICENSE).

![Screenshot of Edit with the About dialog in the foreground](./assets/edit_hero_image.png)

## Fork customizations

On top of upstream [microsoft/edit](https://github.com/microsoft/edit), this fork adds:

- **Configurable key bindings** via a `keybindings.json` file, including support for the
  [Kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/) so combinations
  such as `Ctrl+Shift+<letter>` can be told apart from `Ctrl+<letter>`. See
  [Key bindings](#key-bindings) for the full list of actions and how to remap them.
- **Nano-style "cut current line" action** — `cutLine`, bound to `Ctrl+K` by default.

Everything else tracks upstream.

## Installation

Builds are distributed through my Homebrew tap and the GitHub
[Releases page](https://github.com/mikuta0407/edit/releases/latest).

### macOS / Linux (Homebrew)

```sh
brew install mikuta0407/apps/edit
```

or:

```sh
brew tap mikuta0407/apps
brew install edit
```

Supported: macOS (Apple Silicon), Linux (x86_64 / arm64).

### Windows

Download `edit-<version>-windows-amd64.zip` or `edit-<version>-windows-arm64.zip` from the
[Releases page](https://github.com/mikuta0407/edit/releases/latest), extract it, and run
`edit.exe`.

### Build from source

See [Build Instructions](#build-instructions) below.

## Build Instructions

* [Install Rust](https://www.rust-lang.org/tools/install)
* Clone the repository
* If you're using nightly Rust:
  ```sh
  cargo build --release --config .cargo/release.toml
  ```
* If you're using stable Rust:
  * Ideally: Set the environment variable `RUSTC_BOOTSTRAP=1` and use the **nightly** build instructions above.
    This is recommended, because it drastically reduces the binary size and slightly improves performance.
  * Otherwise, simply run:
    ```sh
    cargo build --release
    ```

### Build Configuration

You can set the following environment variables at build-time to configure the build:

Environment variable | Description
--- | ---
`EDIT_CFG_ICU*` | See [ICU library name (SONAME)](#icu-library-name-soname) below for details. Linux package maintainers are advised to review and configure these options.
`EDIT_CFG_LANGUAGES` | A comma-separated list of languages to include in the build. See [i18n/edit.toml](i18n/edit.toml) for available languages.

## Customization

### Key bindings

You can remap the editor's shortcuts with a `keybindings.json` file in your
configuration directory:

* **Linux / macOS:** `~/.config/edit/keybindings.json`
  (or `$XDG_CONFIG_HOME/edit/keybindings.json`)
* **Windows:** `%APPDATA%\edit\keybindings.json`

The file maps an *action* to a key (or a list of keys). For example, to use the
Emacs-style `Ctrl+A` / `Ctrl+E` for moving to the start/end of the line while
keeping "select all" available on `Ctrl+Shift+A`:

```json
{
  "selectAll": "ctrl+shift+a",
  "lineStart": "ctrl+a",
  "lineEnd": "ctrl+e"
}
```

A key string is written as `[ctrl+][alt+][shift+]<key>` (case-insensitive,
modifier order does not matter), where `<key>` is a letter `a`–`z`, a digit
`0`–`9`, or one of `home`, `end`, `left`, `right`, `up`, `down`, `pageup`,
`pagedown`, `insert`, `delete`, `backspace`, `tab`, `enter`, `escape`, `space`,
or `f1`–`f24`. Assign several keys to one action with an array, e.g.
`"copy": ["ctrl+c", "ctrl+insert"]`.

> **`Ctrl+Shift+<letter>` and the Kitty keyboard protocol.** Classic terminals
> cannot tell `Ctrl+Shift+A` apart from `Ctrl+A` (both send the same byte), so
> bindings like `ctrl+shift+a` only work in terminals that support the
> [Kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/)
> — e.g. Ghostty, kitty, WezTerm, foot, Alacritty, or recent iTerm2. Edit
> detects support at startup and enables it automatically (falling back to the
> classic encoding otherwise, including Terminal.app). This negotiation works
> over SSH too. If you run Edit inside `tmux`/`screen`, the multiplexer must
> forward extended keys (tmux 3.3+ with `set -g extended-keys on`); otherwise
> pick a binding that does not rely on `Ctrl+Shift` (e.g. a free `Ctrl`+letter
> or a function key).

Assigning a key that is already used by another action transfers it to the new
action (the last assignment wins), so the example above frees `Ctrl+A` from
"select all" automatically. Changes take effect the next time you start the
editor.

You can open the file directly from **File ▸ Key Bindings** (it is created for
you if it does not exist yet).

Available actions and their defaults:

Action | Default | Description
--- | --- | ---
`new` | `Ctrl+N` | New file
`open` | `Ctrl+O` | Open file
`save` | `Ctrl+S` | Save
`saveAs` | `Ctrl+Shift+S` | Save as…
`close` | `Ctrl+W` | Close file
`exit` | `Ctrl+Q` | Exit
`goToFile` | `Ctrl+P` | Go to file
`goToLine` | `Ctrl+G` | Go to line
`find` | `Ctrl+F` | Find
`replace` | `Ctrl+R` | Replace
`findNext` | `F3` | Find next
`selectAll` | `Ctrl+A` | Select all
`selectLine` | `Ctrl+L` | Select line
`cutLine` | `Ctrl+K` | Cut the current line (nano-style)
`copy` | `Ctrl+C`, `Ctrl+Insert` | Copy
`cut` | `Ctrl+X`, `Shift+Delete` | Cut
`paste` | `Ctrl+V`, `Shift+Insert` | Paste
`undo` | `Ctrl+Z` | Undo
`redo` | `Ctrl+Y`, `Ctrl+Shift+Z` | Redo
`deleteWordLeft` | `Ctrl+H` | Delete word to the left
`deleteWordRight` | `Ctrl+Delete` | Delete word to the right
`toggleWordWrap` | `Alt+Z` | Toggle word wrap
`toggleOvertype` | `Insert` | Toggle overtype mode
`wordLeft` | `Alt+B` (macOS) | Move one word left
`wordRight` | `Alt+F` (macOS) | Move one word right
`lineStart` | _(unbound)_ | Move to start of line
`lineEnd` | _(unbound)_ | Move to end of line
`documentStart` | _(unbound)_ | Move to start of document
`documentEnd` | _(unbound)_ | Move to end of document

The cursor movement keys (arrows, `Home`, `End`, `Page Up`/`Page Down`) keep
their built-in behavior, including selection with `Shift`. The `lineStart` /
`lineEnd` actions move without selecting; use `Shift+Home` / `Shift+End` to
select.

## Notes to Package Maintainers

### Package Naming

The canonical executable name is "edit" and the alternative name is "msedit".
We're aware of the potential conflict of "edit" with existing commands and recommend alternatively naming packages and executables "msedit".
Names such as "ms-edit" should be avoided.
Assigning an "edit" alias is recommended, if possible.

### ICU library name (SONAME)

This project optionally depends on the ICU library for its Search and Replace functionality.

By default, the project will look for the following library names:

 Variable | Windows | macOS | Linux / Other
----------|---------|-------|---------------
`EDIT_CFG_ICUUC_SONAME` | `icuuc.dll` | `libicucore.dylib` | `libicuuc.so`
`EDIT_CFG_ICUI18N_SONAME` | `icuin.dll` | `libicucore.dylib` | `libicui18n.so`

If your installation uses a different SONAME, please set the following environment variable at build time:
* `EDIT_CFG_ICUUC_SONAME`:
  For instance, `libicuuc.so.76`.
* `EDIT_CFG_ICUI18N_SONAME`:
  For instance, `libicui18n.so.76`.

Additionally, this project assumes that the ICU exports symbols without `_` prefix and without version suffix, such as `u_errorName`.
If your installation uses versioned exports, please set:
* `EDIT_CFG_ICU_CPP_EXPORTS`:
  If set to `true`, it'll look for C++ symbols such as `_u_errorName`.
  Enabled by default on macOS.
* `EDIT_CFG_ICU_RENAMING_VERSION`:
  If set to a version number, such as `76`, it'll look for symbols such as `u_errorName_76`.

Finally, you can set the following environment variables:
* `EDIT_CFG_ICU_RENAMING_AUTO_DETECT`:
  If set to `true`, the executable will try to detect the `EDIT_CFG_ICU_RENAMING_VERSION` value at runtime.
  The way it does this is not officially supported by ICU and as such is not recommended to be relied upon.
  Enabled by default on UNIX (excluding macOS) if no other options are set.

To test your build settings, run `cargo test` with the `--ignored` flag. For instance:
```sh
cargo test -- --ignored
```
