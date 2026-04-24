# RustyTeams - Rust + CEF

Always use Context7 when I need library/API documentation, code generation, setup or configuration steps without me having to explicitly ask.

## Overview
Build a lightweight desktop application in Rust using the Chromium Embedded Framework (CEF) that wraps Microsoft Teams web (https://teams.microsoft.com) as a standalone desktop app.

## Goals
- Full Chromium feature parity (WebRTC, service workers, notifications)
- Lower resource usage than Electron
- Windows-first (Linux/macOS stretch goals)
- Persistent sessions between launches
- Minimal UI — no browser chrome, just the Teams web app

## Tech Stack
- **Language:** Rust
- **Framework:** CEF (Chromium Embedded Framework) via `cef-rs` or direct CEF C API bindings
- **Target Platform:** Windows (WebRTC must work)
- **Build Tool:** Cargo + cmake for CEF integration

## Architecture
- Single-window application loading `https://teams.microsoft.com`
- User-Agent string spoofed to match Microsoft Edge so Teams serves the full experience
- Separate user data directory for persistent cookies, localStorage, and session data
- CEF subprocess model (browser process + render process) as required by CEF

## Required Features

### Window Management
- Single main window, resizable, with a minimum size of 800x600
- Handle popup windows for OAuth authentication flows (login.microsoftonline.com, login.live.com)
- Child popups for meeting pop-outs should open as separate managed windows
- System tray icon with minimize-to-tray behavior

### Navigation & Security
- Lock navigation to an allowlist of Microsoft domains:
  - teams.microsoft.com
  - login.microsoftonline.com
  - login.live.com
  - *.microsoft.com
  - *.office.com
  - *.sharepoint.com
  - *.office365.com
- Block all other external navigation — open in the user's default browser instead

### Authentication
- Handle OAuth redirect flows and popup-based login without breaking
- Persist cookies and session storage across app restarts via a dedicated user data directory at `%LOCALAPPDATA%\TeamsWrapper`

### Permissions
- Auto-allow camera and microphone access for teams.microsoft.com
- Auto-allow notification permission for teams.microsoft.com
- Auto-allow screen sharing/display capture

### Notifications
- Bridge CEF web notification events to Windows native toast notifications
- Clicking a notification should focus/restore the app window

### Downloads
- Implement CEF download handler
- Prompt the user with a native save dialog for file downloads
- Show download progress

### User-Agent
- Set user agent to mimic the latest stable Microsoft Edge on Windows, e.g.:
  `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0`
- Update the version numbers to stay current

### Performance
- Limit JS heap size via CEF command-line switches
- Enable background throttling when the window is minimized or hidden
- Disable unnecessary Chromium features: sync, translate, extensions, background-networking

## CEF Configuration Switches
Pass the following Chromium command-line switches via CEF settings:
- `--disable-extensions`
- `--disable-sync`
- `--disable-translate`
- `--disable-background-networking`
- `--autoplay-policy=no-user-gesture-required`
- `--enable-features=WebRTCPipeWireCapturer` (Linux only)

## CEF Handlers to Implement
- **CefLifeSpanHandler** — manage window creation, popups, and close events
- **CefRequestHandler** — enforce domain allowlist, handle SSL errors
- **CefDisplayHandler** — update window title based on page title
- **CefDownloadHandler** — file download prompts and progress
- **CefPermissionHandler** — auto-grant camera/mic/notifications for Teams
- **CefKeyboardHandler** — handle keyboard shortcuts (Ctrl+Q quit, etc.)

## Build & Packaging
- Bundle the CEF binaries alongside the Rust executable
- Use CEF prebuilt binaries from https://cef-builds.spotifycdn.com/index.html (Standard Distribution)
- Target CEF version should match a recent stable Chromium release
- Final package should be distributable as a single folder or installer (consider WiX or NSIS for Windows installer)
- Include a `README.md` with build instructions

## Directory Structure (suggested)
```
teams-wrapper/
├── Cargo.toml
├── build.rs              # CEF binary download/linking
├── src/
│   ├── main.rs           # Entry point, CEF initialization
│   ├── app.rs            # CefApp implementation
│   ├── browser.rs        # Window creation, navigation
│   ├── handlers/
│   │   ├── life_span.rs
│   │   ├── request.rs
│   │   ├── display.rs
│   │   ├── download.rs
│   │   ├── permission.rs
│   │   └── keyboard.rs
│   ├── notifications.rs  # Windows toast notification bridge
│   └── tray.rs           # System tray icon
├── resources/
│   ├── icon.ico
│   └── icon.png
└── cef/                  # CEF binaries (downloaded at build time)
```

## Out of Scope
- Linux/macOS support (initial release)
- Chrome extension support
- Widevine DRM
- Auto-updates (can be added later)
- Custom theming or CSS injection