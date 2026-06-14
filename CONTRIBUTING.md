# Contributing to Bypass

Thank you for your interest in improving **Bypass**. This guide covers how to set up your environment, what we expect from contributions, and how to propose changes.

## Code of conduct

By participating in this project, you agree to follow our [Code of Conduct](CODE_OF_CONDUCT.md). Please be respectful and constructive in issues, pull requests, and discussions.

## How to contribute

- **Bugs and enhancements**: open an [issue](https://github.com/mdiazn80/bypass/issues) describing current vs expected behavior and steps to reproduce when applicable.
- **Code changes**: open a pull request from a clearly named branch; link the related issue when one exists.
- **Documentation and translations**: improvements to the README or in-app copy are welcome; keep the same clear, technical tone.

## Prerequisites

- [Node.js](https://nodejs.org/) ≥ 22  
- [pnpm](https://pnpm.io/) ≥ 10  
- [Rust](https://www.rust-lang.org/tools/install) (stable)  
- Tauri dependencies for your platform: [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

Optional: [Task](https://taskfile.dev/) for targets defined in `Taskfile.yml`.

## Local setup

```bash
git clone https://github.com/mdiazn80/bypass.git
cd bypass
pnpm install
pnpm tauri dev
```

Production build:

```bash
pnpm tauri build
```

Artifacts are written under `target/release/bundle/`. For local builds without signing or updater artifacts, you can use `task build:local` if Task is installed (see the README).

To set up the signing secrets required for a full release build, follow the instructions in **[docs/release-signing.md](docs/release-signing.md)**.

## Project layout

- **`src/`**: React frontend, TypeScript, Zustand, Vite.
- **`src-tauri/`**: Tauri app crate (commands, hosts file read/write, tray, credential commands).
- **`src-tauri/src/secrets/`**: encrypted credential vault (`SecretBackend` trait + `HybridBackend`, crypto, keystore).

Build and test the Rust crate with:

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # unit tests
pnpm tauri dev                                     # the GUI
```

Before changing sensitive hosts-related behavior, review `src-tauri/src/hosts.rs` and the managed-block markers described in the README. Before changing credential storage, review `src-tauri/src/secrets/` (crypto, keystore, backend) and keep secret values out of logs.

## Style and quality

- **TypeScript / React**: use strong typing and patterns consistent with existing components; run project checks before submitting (for example `pnpm build` for the frontend).
- **Rust**: follow common ecosystem conventions (`cargo fmt`, `cargo clippy` where applicable).
- **Commits**: use clear messages; avoid large, context-free commits.

If your change touches security (privilege elevation, file I/O, IPC), describe that explicitly in the pull request to speed up review.

## Pull requests

1. Branch from `develop`.  
2. Keep changes focused (one feature or fix per PR when practical).  
3. Update docs or comments only where your change requires it.  
4. In the PR, explain what changed and how to test it (OS, manual steps if CI does not cover the case).

Repository CI may run on your PRs; ensure relevant checks pass.

## License

By contributing, you agree that your contributions will be licensed under the same license as the project: **Apache License 2.0** (see [LICENSE](LICENSE)).
