# RustyTeams

Lightweight desktop wrapper for [Microsoft Teams web](https://teams.microsoft.com)
built in Rust on the Chromium Embedded Framework. Feature-complete Chromium
(WebRTC, service workers, notifications) with a much smaller footprint than
Electron.

Windows-first. Linux/macOS are stretch goals.

## Prerequisites

- [Rust](https://rustup.rs) (stable, MSRV = the edition in `Cargo.toml`)
- Visual Studio Build Tools 2022 with the **Desktop development with C++** workload
- [CMake](https://cmake.org/download/) on `PATH`
- **Ninja** — required by `cef-dll-sys`'s CMake build.
  Install with `winget install Ninja-build.Ninja`, or via the VS Installer's
  *"C++ CMake tools for Windows"* component (bundles both CMake and Ninja).
- Python 3 on `PATH` (used by CEF's build scripts transitively)

Run `make doctor` after installing to verify every tool is visible.

## 1. Fetch the CEF binaries

The `cef` crate expects a CEF Standard Distribution to be unpacked somewhere
and pointed at via the `CEF_PATH` environment variable. The easiest route is
the `export-cef-dir` helper from the `cef-rs` workspace:

```powershell
git clone https://github.com/tauri-apps/cef-rs
cd cef-rs
cargo run -p export-cef-dir -- --force $env:USERPROFILE/.local/share/cef
```

Then set the env vars for every shell that builds or runs this app:

```powershell
$env:CEF_PATH = "$env:USERPROFILE/.local/share/cef"
$env:PATH     = "$env:PATH;$env:CEF_PATH"
```

The build links against `libcef.dll.lib` from `$CEF_PATH/Release/` and the run
time needs the same directory on `PATH` so `libcef.dll` and the ICU data file
can be located.

## 2. Build

```powershell
cd rustyteams
cargo build --release
```

## 3. Run

`libcef.dll`, `*.bin` snapshot files, and the `Resources/` directory must sit
next to `rustyteams.exe` (or be reachable via `PATH`). During development the
`CEF_PATH` shortcut works; for a shippable install, copy them into the release
folder:

```powershell
Copy-Item $env:CEF_PATH/Release/* target/release/
Copy-Item -Recurse $env:CEF_PATH/Resources/* target/release/
./target/release/rustyteams.exe
```

## Makefile shortcuts

From a POSIX shell (git-bash, MSYS, WSL):

```bash
make doctor        # check toolchain + CEF_PATH
make setup         # clone cef-rs and export CEF into $CEF_PATH
make run           # debug build, sync CEF runtime, launch
make run-release   # same, release profile
make package       # stage dist/rustyteams/
make dist          # produce dist/rustyteams.zip
make clippy fmt test
```

Override `CEF_PATH` per-invocation if you keep CEF elsewhere:
`make CEF_PATH=D:/cef/146 run`.

## What's in the box

| Area                | Status |
| ------------------- | ------ |
| Main window (Views) | ✅ `src/browser.rs` — 1280×800 default, 800×600 minimum, Alloy runtime |
| Domain allowlist    | ✅ `src/handlers/request.rs`, `life_span.rs` — outside links open in the configured external browser (or OS default) |
| External browser override | ✅ `src/config.rs`, tray menu "Set external browser…" — persisted at `external_browser.txt` |
| OAuth / meeting popups | ✅ `src/handlers/life_span.rs` — allowlisted popups become managed child windows |
| Camera / mic / screen | ✅ `src/handlers/permission.rs` — auto-granted for Microsoft origins |
| Downloads            | ✅ `src/handlers/download.rs` — native Save dialog via `rfd` |
| Keyboard            | ✅ `src/handlers/keyboard.rs` — Ctrl+Q quit, F12 DevTools, Ctrl+0 reset zoom |
| Title sync          | ✅ `src/handlers/display.rs` |
| Persistent session  | ✅ `%LOCALAPPDATA%\io.alexieff.rustyteams` (cookies + cache) |
| Edge user-agent     | ✅ `src/browser.rs` — Chrome/Edg 146 |
| Command-line switches | ✅ `src/app.rs` — disables extensions, sync, translate, WebUSB/Bluetooth/MediaRouter |
| System tray         | ✅ `src/tray.rs` — Show / external-browser / Quit |
| Minimize-to-tray    | ✅ `src/browser.rs` — closing the window hides to tray, Quit does a real shutdown |
| AppUserModelID      | ✅ `src/main.rs` — Windows groups the taskbar icon + toasts |
| Installer           | ✅ `installer.nsi` — `make installer` (needs `makensis` on PATH) |
| Toast notifications | ⚠️ `src/notifications.rs` — dispatch ready, Web-Notification → toast bridge (render-process handler + IPC) still TODO |
| Icon                | ❌ Drop a real `resources/icon.ico` to replace the placeholder |

## Directory layout

```
rustyteams/
├── Cargo.toml
├── build.rs                      # winres: embeds manifest + icon
├── resources/
│   └── rustyteams.exe.manifest   # DPI awareness, long-path support, UTF-8
├── src/
│   ├── main.rs                   # Entry point, subprocess dispatch, CEF init
│   ├── app.rs                    # CefApp + command-line switches + context init
│   ├── browser.rs                # Window + BrowserView + allowlist helpers
│   ├── handlers/
│   │   ├── mod.rs                # CefClient wiring
│   │   ├── display.rs
│   │   ├── download.rs
│   │   ├── keyboard.rs
│   │   ├── life_span.rs
│   │   ├── permission.rs
│   │   └── request.rs
│   ├── notifications.rs          # Windows toast dispatch
│   └── tray.rs                   # System tray
```

## Out of scope (for now)

- Linux / macOS packaging
- Chrome extensions
- Widevine DRM
- Auto-updates
