# Bypass

A macOS hosts file manager built with [Tauri 2](https://tauri.app/), React and TypeScript.

Bypass lets you create, organize and toggle groups of hosts entries (called **Contexts**) without manually editing `/etc/hosts`. Each context can be enabled or disabled independently, and the system hosts file is updated automatically with elevated privileges.

## Features

- **Contexts** &mdash; Group related hosts entries and toggle them on/off with a single click.
- **Credential contexts** &mdash; Manage sensitive environment variables (API keys, tokens, connection strings) in an encrypted vault (OS keychain master key + ChaCha20-Poly1305 on-disk store). Values can reference each other with `{$VAR}`.
- **Syntax highlighting** &mdash; IPs, hostnames and comments are color-coded in the editor.
- **System Hosts view** &mdash; Read-only preview of the current `/etc/hosts` file, auto-refreshed when contexts change.
- **Import / Export** &mdash; Share contexts as JSON files using native OS file dialogs.
- **Touch ID support** &mdash; Biometric authentication to authorize hosts file changes (no double prompt).
- **System tray** &mdash; Minimize to tray and keep running in the background.
- **Autostart** &mdash; Optionally launch at login.

## Screenshots

<!-- TODO: add screenshots -->

## Getting Started

### Prerequisites

- macOS (Apple Silicon)
- [Node.js](https://nodejs.org/) >= 22
- [pnpm](https://pnpm.io/) >= 10
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Tauri macOS dependencies &mdash; see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

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

Binaries are generated in `target/release/bundle/`.

### Taskfile

If you use [Task](https://taskfile.dev/) (`go-task`), common commands are wrapped in `Taskfile.yml` at the repo root. Install Task, then run `task` or `task --list` to see all targets.

| Task | Command | Description |
|------|---------|-------------|
| List tasks | `task` | Prints the same list as `task --list`. |
| Install | `task install` | Runs `pnpm install`. |
| Develop | `task dev` | Starts Vite and Tauri in dev mode (`pnpm tauri dev`). |
| Build (release) | `task build` | Full `pnpm tauri build`. Requires `TAURI_SIGNING_PRIVATE_KEY` when `createUpdaterArtifacts` is enabled in `src-tauri/tauri.conf.json`. |
| Build (local) | `task build:local` | Release build without code signing and without updater artifacts (`--no-sign` and `createUpdaterArtifacts: false` for that run). Use when you do not have signing keys locally. |
| Frontend only | `task build:frontend` | Runs `pnpm build` (TypeScript + Vite); does not invoke Tauri. |
| Preview | `task preview` | Serves the Vite production build (`pnpm preview`); no native shell. |
| Clean | `task clean` | Deletes `dist/` and runs `cargo clean` for `src-tauri`. |
| Clean all | `task clean:all` | Runs `task clean`, then removes `node_modules/`. |
| Tauri info | `task info` | Runs `pnpm tauri info` (toolchain and environment summary). |

The `clean` tasks use `rm -rf` and work as-is on macOS.

## Project Structure

```
bypass/
├── src/                        # Frontend (React + TypeScript)
│   ├── components/
│   │   ├── TopBar.tsx          # Navigation: Hosts / Credentials / Settings
│   │   ├── Sidebar.tsx         # Hosts context list, toggle, export
│   │   ├── ContextEditor.tsx   # Hosts editor with syntax highlighting
│   │   ├── FileDropZone.tsx    # Drag-and-drop file import for new contexts
│   │   ├── CredentialSidebar.tsx  # Credential context list
│   │   ├── CredentialEditor.tsx   # Key/value editor with masked values
│   │   ├── Settings.tsx        # App settings (autostart, tray, etc.)
│   │   ├── AboutModal.tsx      # About dialog
│   │   └── Footer.tsx          # Version and GitHub link
│   ├── stores/
│   │   ├── useContextStore.ts     # Hosts contexts state (Zustand)
│   │   ├── useCredentialStore.ts  # Credential contexts state (Zustand)
│   │   └── useConfigStore.ts      # App config state
│   └── services/
│       └── tauri.ts            # Tauri IPC bindings
├── src-tauri/                  # Tauri app crate (Rust)
│   ├── Cargo.toml              # Crate manifest
│   ├── src/
│   │   ├── lib.rs              # App setup and plugin registration
│   │   ├── commands.rs         # Hosts/config Tauri commands
│   │   ├── credentials.rs      # Credential vault Tauri commands
│   │   ├── hosts.rs            # Hosts file read/write/merge logic
│   │   ├── biometric.rs        # Touch ID authentication (macOS)
│   │   ├── tray.rs             # System tray menu
│   │   ├── models.rs           # Data models (Context, AppConfig)
│   │   ├── storage.rs          # JSON file persistence
│   │   ├── state.rs            # Shared app state
│   │   └── secrets/            # Encrypted credential vault
│   │       ├── backend.rs      # SecretBackend trait + HybridBackend
│   │       ├── crypto.rs       # ChaCha20-Poly1305 seal/open
│   │       ├── interpolate.rs  # `{$VAR}` references between variables
│   │       ├── keystore.rs     # OS keychain master key + env fallback
│   │       └── vault.rs        # High-level API
│   └── tauri.conf.json         # Tauri configuration
└── .github/
    └── workflows/
        └── version-tag-and-binary.yml  # CI: build binaries on merged PRs
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
- **Import**: click `+` to create a context, then drag a hosts-format file onto the drop zone (or click it to browse). The dropped file initializes the context content.

Export JSON format:

```json
{
  "name": "My Context",
  "content": "127.0.0.1  example.local\n192.168.1.1  api.local"
}
```

## Credential contexts

Bypass can manage groups of sensitive environment variables (a *credential context*) without writing `.env` files or storing secrets in plaintext.

### Storage (hybrid backend)

- A random 32-byte master key is generated on first use and stored in the **macOS Keychain**.
- Contexts and variables are stored in an encrypted file (`store.enc`) next to the app data, sealed with **ChaCha20-Poly1305** using that master key.
- Secret values are never written to disk in plaintext, and the master key never appears in the file.

In the GUI, open the **Credentials** tab to create contexts and add/edit/delete variables (values are masked by default with a reveal toggle). You can initialize a context by dragging a `.env`-style file onto the drop zone; keys without a value are imported empty so you can fill them in manually.

### Referencing other variables

A value can reuse another variable of the **same** context with `{$VAR}`:

| Variable | Stored value | Exported value |
|----------|--------------|----------------|
| `APP_PATH` | `/opt/app` | `/opt/app` |
| `APP_PATH_CONFIG` | `{$APP_PATH}/config` | `/opt/app/config` |
| `APP_LOGS` | `{$APP_PATH_CONFIG}/logs` | `/opt/app/config/logs` |

- References are resolved **when the value is handed to your shell**, not when it is saved. The vault keeps the template, so editing `APP_PATH` updates every variable derived from it on the next prompt.
- References may chain to any depth and are order-independent.
- Only the two-character sequence `{$` starts a reference. A bare `$` is always literal, so secrets containing dollar signs (bcrypt hashes, passwords) need no escaping. Write `\{$` for a literal `{$`.
- References to shell variables are **not** supported: `$HOME` and `${HOME}` are exported verbatim.
- A reference that is undefined, cyclic (`A → B → A`) or too deeply nested is exported verbatim and flagged in the editor, under the variable it affects.

### Security notes

- Secret values are never written to disk in plaintext, and the master key never appears in the encrypted store.

## CI / CD

A GitHub Actions workflow (`.github/workflows/version-tag-and-binary.yml`) builds binaries automatically when a pull request from `develop` is merged into `main`. It produces a signed and notarized macOS release, along with a `latest.json` updater manifest.

For a step-by-step guide on generating the key pair and obtaining every required secret, see **[docs/release-signing.md](docs/release-signing.md)**.

| Platform | Architecture | Artifacts |
|----------|-------------|-----------|
| macOS    | ARM64 (Apple Silicon) | `.dmg`, `.app.tar.gz` |

## Tech Stack

- **Frontend**: React 19, TypeScript, Zustand, Vite
- **Backend**: Rust, Tauri 2
- **Crypto**: ChaCha20-Poly1305, macOS Keychain (`keyring`)
- **Plugins**: `tauri-plugin-dialog`, `tauri-plugin-fs`, `tauri-plugin-opener`, `tauri-plugin-autostart`

## License

This project is licensed under the Apache License 2.0 &mdash; see the [LICENSE](LICENSE) file for details.
