# Bypass

A lightweight, cross-platform hosts file manager built with [Tauri 2](https://tauri.app/), React and TypeScript.

Bypass lets you create, organize and toggle groups of hosts entries (called **Contexts**) without manually editing `/etc/hosts`. Each context can be enabled or disabled independently, and the system hosts file is updated automatically with elevated privileges.

## Features

- **Contexts** &mdash; Group related hosts entries and toggle them on/off with a single click.
- **Syntax highlighting** &mdash; IPs, hostnames and comments are color-coded in the editor.
- **System Hosts view** &mdash; Read-only preview of the current `/etc/hosts` file, auto-refreshed when contexts change.
- **Import / Export** &mdash; Share contexts as JSON files using native OS file dialogs.
- **Touch ID support** &mdash; Biometric authentication on macOS to authorize hosts file changes (no double prompt).
- **System tray** &mdash; Minimize to tray and keep running in the background.
- **Autostart** &mdash; Optionally launch at login.
- **Cross-platform** &mdash; macOS (ARM & Intel), Linux and Windows.

## Screenshots

<!-- TODO: add screenshots -->

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) >= 22
- [pnpm](https://pnpm.io/) >= 10
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Platform-specific Tauri dependencies &mdash; see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Install

```bash
git clone https://github.com/mdiazn80/bypass.git
cd bypass
pnpm install
```

### Development

```bash
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

Binaries are generated in `src-tauri/target/release/bundle/`.

## Project Structure

```
bypass/
├── src/                        # Frontend (React + TypeScript)
│   ├── components/
│   │   ├── TopBar.tsx          # Navigation bar with About button
│   │   ├── Sidebar.tsx         # Context list, toggle, import/export
│   │   ├── ContextEditor.tsx   # Hosts editor with syntax highlighting
│   │   ├── Settings.tsx        # App settings (autostart, tray, etc.)
│   │   ├── AboutModal.tsx      # About dialog
│   │   └── Footer.tsx          # Version and GitHub link
│   ├── stores/
│   │   ├── useContextStore.ts  # Contexts state management (Zustand)
│   │   └── useConfigStore.ts   # App config state management
│   └── services/
│       └── tauri.ts            # Tauri IPC bindings
├── src-tauri/                  # Backend (Rust)
│   ├── src/
│   │   ├── lib.rs              # App setup and plugin registration
│   │   ├── commands.rs         # Tauri commands (IPC handlers)
│   │   ├── hosts.rs            # Hosts file read/write/merge logic
│   │   ├── biometric.rs        # Touch ID authentication (macOS)
│   │   ├── tray.rs             # System tray menu
│   │   ├── models.rs           # Data models (Context, AppConfig)
│   │   ├── storage.rs          # JSON file persistence
│   │   └── state.rs            # Shared app state
│   └── tauri.conf.json         # Tauri configuration
└── .github/
    └── workflows/
        └── release.yml         # CI: build binaries on merged PRs
```

## How It Works

1. You create **Contexts**, each containing one or more hosts entries (e.g. `127.0.0.1 mysite.local`).
2. When you **enable** a context, Bypass reads the system hosts file, appends a managed block with your entries, and writes it back using elevated privileges.
3. When you **disable** a context, the managed block is updated to remove those entries.
4. The managed section is delimited by markers so Bypass never touches your existing hosts entries.

### Managed block example

```
# ===== BYPASS MANAGED START =====
# >>> Context: "Development"
127.0.0.1  api.local
127.0.0.1  app.local
# <<< Context: "Development"
# ===== BYPASS MANAGED END =====
```

## Import / Export

- **Export**: click the download arrow on any context in the sidebar. A native Save dialog lets you choose where to save the `.json` file.
- **Import**: click the `+` button, then **Import**. A native Open dialog lets you pick a `.json` file (single context) or an array of contexts.

JSON format:

```json
{
  "name": "My Context",
  "content": "127.0.0.1  example.local\n192.168.1.1  api.local"
}
```

## CI / CD

A GitHub Actions workflow (`.github/workflows/release.yml`) builds binaries automatically when a pull request to `main` is merged. It produces artifacts for:

| Platform | Architecture | Artifacts |
|----------|-------------|-----------|
| macOS    | ARM64       | `.dmg`, `.app` |
| macOS    | x86_64      | `.dmg`, `.app` |
| Linux    | x86_64      | `.deb`, `.AppImage` |
| Windows  | x86_64      | `.exe` (NSIS), `.msi` |

## Tech Stack

- **Frontend**: React 19, TypeScript, Zustand, Vite
- **Backend**: Rust, Tauri 2
- **Plugins**: `tauri-plugin-dialog`, `tauri-plugin-fs`, `tauri-plugin-opener`, `tauri-plugin-autostart`

## License

This project is licensed under the Apache License 2.0 &mdash; see the [LICENSE](LICENSE) file for details.
