# triSig

- English: this file
- 中文: [README-ZNH.md](README-ZNH.md)

A lightweight desktop traffic light status indicator that displays red, yellow, and green signal lights in the top-right corner of the screen (configurable). Supports remote status switching and blinking effects via HTTP API.

## Use Cases

- **Claude Code / Automation Tool Status**: Switch light colors via HTTP requests in Claude Code hooks or scripts to visually indicate current work status (e.g., processing / waiting for input / idle). See [docs/claude_code.md](docs/claude_code.md) for a local hook example.
- **CI/CD Pipeline Monitoring**: Build scripts send requests at key stages for instant desktop feedback on pipeline progress
- **Local Service Health Check**: Periodically request a health endpoint — red light blinks when the service is down
- **Pomodoro / Focus Mode**: Control light transitions via scripts as a distraction-free visual signal

## System Architecture

```
┌─────────────┐   HTTP GET    ┌──────────────────┐   Tauri Event   ┌─────────────────┐
│  External    │ ────────────> │  tiny_http Server │ ─────────────> │  Vue 3 Frontend │
│  Tools       │              │  (port 53789+)    │               │  (transparent    │
│ (curl/script)│              └──────────────────┘               │   overlay window)│
└─────────────┘                     │                            └─────────────────┘
                                    │ Embedded
                                    ▼
                             ┌──────────────────┐
                             │   Tauri v2 App   │
                             │  (Rust + WebView)│
                             └──────────────────┘
```

- **Frontend (Vue 3 + Vite)**: Renders a transparent, borderless window with red, yellow, and green lamps; supports solid and blinking modes
- **HTTP Server (tiny_http)**: Embedded in the Tauri app, listens on the LAN IP, receives GET requests and forwards them as Tauri events
- **Tauri v2 Tray**: System tray menu for switching displays, toggling corners, viewing/copying server addresses, and quitting

Window features:
- 100×180px transparent borderless floating window, always on top, hidden from the taskbar
- Defaults to the top-right corner of the primary display; can be switched to top-left or other displays via the tray menu
- Supports macOS multi-display; automatically selects the largest display as the default

## API

The HTTP server listens on the LAN address after startup. The port is auto-assigned starting from `53789`.

```
GET /?color={red|yellow|green}&interval={milliseconds}
```

| Parameter  | Required | Description |
|------------|----------|-------------|
| `color`    | Yes      | Light color: `red`, `yellow`, `green` |
| `interval` | Yes      | Blink interval in milliseconds; use `0` for solid (no blink) |

Examples:

```bash
# Solid green light
curl "http://127.0.0.1:53789/?color=green&interval=0"

# Red light blinking (500ms interval)
curl "http://127.0.0.1:53789/?color=red&interval=500"

# Solid yellow light
curl "http://127.0.0.1:53789/?color=yellow&interval=0"
```

Click any network address in the tray menu to copy it to the clipboard.

## Build

### Prerequisites

- [Rust](https://www.rust-lang.org/) (stable)
- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- macOS: Xcode Command Line Tools
- Windows: Microsoft Visual Studio C++ Build Tools
- Linux: `webkit2gtk-4.1`, `libayatana-appindicator3` and other system libraries (see [Tauri docs](https://v2.tauri.app/start/prerequisites/))

### Development

```bash
# Install dependencies
pnpm install

# Start dev server (with hot reload)
pnpm tauri dev
```

### Production Build

```bash
pnpm tauri build
```

Build artifacts are located in `src-tauri/target/release/bundle/`, containing platform-specific installers (`.dmg` for macOS, `.msi`/`.exe` for Windows, `.deb`/`.AppImage` for Linux).

## Supported Platforms

| Platform | Status |
|----------|--------|
| macOS    | Full support |
| Windows  | Full support |
| Linux    | Full support |

Built with [Tauri v2](https://v2.tauri.app/), leveraging each platform's native WebView for rendering (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux).

## Tech Stack

- **Desktop Framework**: Tauri v2 (Rust)
- **Frontend**: Vue 3 + Vite
- **HTTP Server**: tiny_http 0.12
- **Event Communication**: Tauri Event System (Rust → WebView)
