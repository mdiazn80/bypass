# Bypass

A macOS hosts file manager built with [Tauri 2](https://tauri.app/), React and TypeScript.

Bypass lets you create, organize and toggle groups of hosts entries (called **Contexts**) without manually editing `/etc/hosts`. Each context can be enabled or disabled independently, and the system hosts file is updated automatically with elevated privileges.

## Features

- **Contexts** &mdash; Group related hosts entries and toggle them on/off with a single click. Reorder them to decide which entries come first in the hosts file.
- **Credential contexts** &mdash; Manage sensitive environment variables (API keys, tokens, connection strings) in an encrypted vault (OS keychain master key + ChaCha20-Poly1305 on-disk store). Several contexts can be active at once, layered by priority, and values can reference each other with `{$VAR}`.
- **Syntax highlighting** &mdash; IPs, hostnames and comments are color-coded in the editor.
- **System Hosts / Active Variables views** &mdash; Read-only previews of the current `/etc/hosts` file and of the merged variables your shells receive, auto-refreshed when contexts change.
- **Import / Export** &mdash; Import hosts or `.env` files by drag-and-drop or from the clipboard; export any screen as a plain hosts or `KEY=value` file using native OS file dialogs.
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
│   │   ├── Sidebar.tsx         # Hosts context list: toggle, reorder, delete
│   │   ├── ContextEditor.tsx   # Hosts editor with syntax highlighting
│   │   ├── FileDropZone.tsx    # Drag-and-drop file import for new contexts
│   │   ├── CredentialSidebar.tsx  # Credential context list: activate, reorder, delete
│   │   ├── CredentialEditor.tsx   # Key/value editor + Active Variables summary
│   │   ├── MoveButtons.tsx     # Up/down chevrons used to reorder sidebar items
│   │   ├── ExportButton.tsx    # Header button that saves a screen to a file
│   │   ├── ConfirmModal.tsx    # Confirmation dialog for destructive actions
│   │   ├── Settings.tsx        # App settings (autostart, tray, etc.)
│   │   ├── AboutModal.tsx      # About dialog
│   │   └── Footer.tsx          # Version and GitHub link
│   ├── stores/
│   │   ├── useContextStore.ts     # Hosts contexts state (Zustand)
│   │   ├── useCredentialStore.ts  # Credential contexts state (Zustand)
│   │   └── useConfigStore.ts      # App config state (active contexts, shell agent)
│   ├── services/
│   │   └── tauri.ts            # Tauri IPC bindings
│   └── utils/
│       └── envFormat.ts        # `.env` serialization for exports
├── src-tauri/                  # Tauri app crate (Rust)
│   ├── Cargo.toml              # Crate manifest
│   ├── src/
│   │   ├── lib.rs              # App setup and plugin registration
│   │   ├── commands.rs         # Hosts/config Tauri commands
│   │   ├── credentials.rs      # Credential vault Tauri commands
│   │   ├── agent.rs            # Local socket agent that serves variables to shells
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
│   │       ├── merge.rs        # Layers active contexts by priority
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
5. Contexts appear in the managed block in **sidebar order**. Use the ▲▼ chevrons next to a context to move it; resolvers take the first matching line, so a higher context wins when two define the same hostname. Reordering re-applies the hosts file if any context is enabled.

While a context is **active** its editor is read-only: every edit to an active context rewrites the system hosts file and asks for administrator credentials, so typing would prompt on every keystroke. Deactivate it to edit, then activate it again. While a toggle is being applied the sidebar and status badge show a spinner &mdash; the administrator prompt can take a few seconds to appear.

Deleting a context (hosts or credentials) always asks for confirmation first.

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

- **Export**: every screen has an **Export** button in its header that opens a native Save dialog. The file is written in the format the screen represents, so it can be used directly or re-imported:

  | Screen | Suggested name | Format |
  |--------|----------------|--------|
  | Hosts → context | `<name>.hosts` | The context's entries, as a plain hosts file |
  | Hosts → System Hosts | `hosts` | A copy of the current `/etc/hosts` |
  | Credentials → context | `<name>.env` | `KEY=value` lines with values as stored (`{$VAR}` references kept) |
  | Credentials → Active Variables | `active.env` | `KEY=value` lines with resolved values, exactly what shells receive |

  In `.env` exports a value is double-quoted (with `\`, `"` and newlines escaped) only when it contains whitespace, `#`, quotes or other characters that would not survive a bare assignment.

- **Import**: click `+` to create a context, then drag a file onto the drop zone (or click it to browse): a hosts-format file for a hosts context, a `.env`-style file for a credential context. The dropped file initializes the context content.

## Credential contexts

Bypass can manage groups of sensitive environment variables (a *credential context*) without writing `.env` files or storing secrets in plaintext.

### Storage (hybrid backend)

- A random 32-byte master key is generated on first use and stored in the **macOS Keychain**.
- Contexts and variables are stored in an encrypted file (`store.enc`) next to the app data, sealed with **ChaCha20-Poly1305** using that master key.
- Secret values are never written to disk in plaintext, and the master key never appears in the file.

In the GUI, open the **Credentials** tab to create contexts and add/edit/delete variables. You can initialize a context by dragging a `.env`-style file onto the drop zone; keys without a value are imported empty so you can fill them in manually.

### Pasting from the clipboard

With a context open, `KEY=value` text on the clipboard can be added directly to it, one or several lines at a time (`export` prefixes, comments and quotes are handled like a `.env` import):

- click **Paste** next to the *Add* button, or
- press `Cmd/Ctrl+V` in the **KEY** field, or anywhere in the editor when no text field is focused.

Text without a `KEY=value` line is ignored, so pasting unrelated content never creates junk keys.

### Active contexts and priority

Any number of contexts can be **active** at once (toggle in the sidebar). Your shells receive the union of their variables, layered by priority:

- The sidebar order is the priority order &mdash; use the ▲▼ chevrons to change it.
- When several active contexts define the same variable, the one **higher** in the sidebar wins; the lower definitions are shadowed and not exported.
- The **Active Variables** item at the top of the sidebar shows the merged result: each variable with its resolved value, the context it comes from, and which contexts it overrides. This is exactly what the shell agent serves.

The active set and the order are stored in the app config (`active_contexts`, `credential_order`); configs from versions with a single `active_context` are migrated automatically.

### Referencing other variables

A value can reuse another variable with `{$VAR}`. References are resolved against the **merged active set**, so a variable in one context can reference one defined in another active context (and always sees the winning definition):

| Variable | Stored value | Exported value |
|----------|--------------|----------------|
| `APP_PATH` | `/opt/app` | `/opt/app` |
| `APP_PATH_CONFIG` | `{$APP_PATH}/config` | `/opt/app/config` |
| `APP_LOGS` | `{$APP_PATH_CONFIG}/logs` | `/opt/app/config/logs` |

- References are resolved **when the value is handed to your shell**, not when it is saved. The vault keeps the template, so editing `APP_PATH` updates every variable derived from it on the next prompt.
- References may chain to any depth and are order-independent.
- Only the two-character sequence `{$` starts a reference. A bare `$` is always literal, so secrets containing dollar signs (bcrypt hashes, passwords) need no escaping. Write `\{$` for a literal `{$`.
- References to shell variables are **not** supported: `$HOME` and `${HOME}` are exported verbatim.
- A reference that is undefined, cyclic (`A → B → A`) or too deeply nested is exported verbatim and flagged in the editor, under the variable it affects. Inside a single context's editor, references are checked against that context only; the Active Variables view checks them against the merged set.

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
