# RustyTeams

Lightweight desktop wrapper for [Microsoft Teams web](https://teams.microsoft.com)
built in Rust on the Chromium Embedded Framework. Feature-complete Chromium
(WebRTC, service workers, notifications) with a much smaller footprint than
Electron.

Supported platforms: **Windows** and **Linux**. macOS is out of scope.

## Prerequisites

### Common
- [Rust](https://rustup.rs) (stable)
- [CMake](https://cmake.org/download/) on `PATH`
- **Ninja** (required by `cef-dll-sys`'s CMake build)
- Python 3 on `PATH` (used transitively by CEF's build scripts)

### Windows
- Visual Studio Build Tools 2022 with the **Desktop development with C++** workload
- Install Ninja via `winget install Ninja-build.Ninja`, or via the VS Installer's
  *"C++ CMake tools for Windows"* component (which bundles both CMake and Ninja).

### Linux
- `build-essential`, `pkg-config`
- GTK + tray-icon deps: `libgtk-3-dev`, `libayatana-appindicator3-dev`, `libxdo-dev`
- CEF Chromium runtime deps: `libnss3`, `libatk1.0-0`, `libatk-bridge2.0-0`,
  `libdrm2`, `libxkbcommon0`, `libxcomposite1`, `libxdamage1`, `libxfixes3`,
  `libxrandr2`, `libgbm1`, `libasound2t64` (or `libasound2`), `libcups2`,
  `libxshmfence1`, `libpango-1.0-0`
- Ninja: `sudo apt install ninja-build`

Run `make doctor` after installing to verify every tool is visible.

## Quick start

```bash
make setup         # fetch CEF Standard Distribution (~400 MB, one-time)
make run           # debug build + launch
```

`make setup` clones the `cef-rs` workspace under `.cache/cef-rs/` and runs its
`export-cef-dir` helper to unpack a CEF Standard Distribution into
`$CEF_PATH` (defaults to `$HOME/.local/share/cef` on Linux,
`$USERPROFILE/.local/share/cef` on Windows). Override with
`make CEF_PATH=/somewhere/else setup`.

Subsequent builds copy `libcef.so` / `libcef.dll` plus every CEF runtime
resource next to the built executable automatically — no `PATH` or
`LD_LIBRARY_PATH` munging required.

## Makefile targets

From any POSIX shell (bash on Linux, git-bash / MSYS / WSL on Windows):

```
make doctor                check toolchain + CEF_PATH
make setup                 clone cef-rs and export CEF into $CEF_PATH
make build / run           debug build (+ launch)
make release / run-release release build (+ launch)
make fmt clippy test       hygiene
make package               stage dist/rustyteams/
make dist                  .zip on Windows, .tar.gz on Linux
make installer             NSIS .exe on Windows, .deb on Linux
make clean / clean-all
```

## DevTools

In-app DevTools is off (it crashes CEF's Alloy runtime when parented to a
BrowserView). The workaround is Chromium's remote debugger: set
`RUSTYTEAMS_DEBUG_PORT=9222` in the environment, launch RustyTeams, then open
`http://localhost:9222` in Chrome or Edge.

## What's in the box

| Area                  | Status |
| --------------------- | ------ |
| Main window (Views)   | ✅ `src/browser.rs` — 1280×800 default, 800×600 minimum, Alloy runtime |
| Domain allowlist      | ✅ `src/handlers/request.rs`, `life_span.rs` — outside links open in the configured external browser (or OS default) |
| External browser override | ✅ `src/config.rs`, `src/external.rs`, tray menu *"Set external browser…"* — persisted at `external_browser.txt` |
| OAuth / meeting popups | ✅ `src/handlers/life_span.rs` — allowlisted popups become managed child windows |
| Camera / mic / screen | ✅ `src/handlers/permission.rs` — auto-granted for Microsoft origins |
| Downloads             | ✅ `src/handlers/download.rs` — native Save dialog via `rfd` |
| Keyboard              | ✅ `src/handlers/keyboard.rs` — Ctrl+Q quit, Ctrl+0 reset zoom *(Windows only; Linux port pending)* |
| Title sync            | ✅ `src/handlers/display.rs` |
| Persistent session    | ✅ per-user data dir (`%LOCALAPPDATA%\io.alexieff.rustyteams` on Windows, `~/.local/share/io.alexieff.rustyteams` on Linux) |
| Edge user-agent       | ✅ `src/browser.rs` — Chrome/Edg 146 |
| Command-line switches | ✅ `src/app.rs` — disables extensions, sync, translate, WebUSB/Bluetooth/MediaRouter |
| System tray           | ✅ `src/tray.rs` — Show / external-browser / Quit |
| Minimize-to-tray      | ✅ `src/browser.rs` — closing the window hides to tray, Quit does a real shutdown |
| AppUserModelID        | ✅ `src/main.rs` — Windows groups the taskbar icon + toasts |
| Toast notifications   | ✅ `src/handlers/render_process.rs` + `src/notifications.rs` — Notification API polyfill bridges to native Windows toasts via IPC *(Linux toast backend TODO)* |
| Packaging — Windows   | ✅ `installer.nsi` (`make installer`, needs `makensis`) + `make dist` zip |
| Packaging — Linux     | ✅ `cargo-deb` via `[package.metadata.deb]` in `Cargo.toml` (`make deb`) + `make dist` tarball |
| CI / Releases         | ✅ `.github/workflows/` — push/PR linting on both platforms, tag-triggered release pipeline |

## CI & releases

Two GitHub Actions workflows drive the project:

- **`ci.yml`** — runs on every push and PR to `main`. `rustfmt` check on
  Ubuntu, plus a `clippy` + `cargo test` matrix on `windows-latest` and
  `ubuntu-latest`. The CEF Standard Distribution (~400 MB) is cached between
  runs keyed on `Cargo.lock`.
- **`release.yml`** — runs on any `v*` tag push (or manual dispatch). Builds
  the Windows `.zip` + NSIS `.exe` installer and the Linux `.tar.gz` + `.deb`
  in parallel, and — on tag pushes only — publishes a GitHub Release with all
  four artifacts attached.

To cut a release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## Directory layout

```
rustyteams/
├── Cargo.toml
├── Makefile                         # cross-platform build / package targets
├── build.rs                         # winres: embeds manifest + icon (Windows only)
├── installer.nsi                    # NSIS definition for the Windows installer
├── .github/workflows/
│   ├── ci.yml
│   └── release.yml
├── resources/
│   ├── rustyteams.exe.manifest      # DPI awareness, long-path support, UTF-8
│   ├── rustyteams.desktop           # Linux desktop entry (installed by .deb)
│   ├── rustyteams.sh                # Linux launcher (installed as /usr/bin/rustyteams)
│   ├── icon.ico                     # Windows app icon
│   ├── icon.svg
│   └── icon-{16,32,48,64,128,256}.png
└── src/
    ├── main.rs                      # Entry point, subprocess dispatch, CEF init
    ├── app.rs                       # CefApp + command-line switches + context init
    ├── browser.rs                   # Window + BrowserView + allowlist helpers
    ├── config.rs                    # Persisted per-user settings
    ├── external.rs                  # Open non-Teams URLs in configured/default browser
    ├── notifications.rs             # Native toast dispatch (Windows)
    ├── tray.rs                      # System tray + menu
    └── handlers/
        ├── mod.rs                   # CefClient wiring + IPC dispatch
        ├── display.rs
        ├── download.rs
        ├── keyboard.rs              # Windows-only (uses sys::MSG)
        ├── life_span.rs
        ├── permission.rs
        ├── render_process.rs        # Notification API polyfill (render process)
        └── request.rs
```

## Out of scope (for now)

- macOS
- Chrome extensions
- Widevine DRM
- Auto-updates
