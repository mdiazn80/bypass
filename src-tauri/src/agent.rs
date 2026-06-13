//! Local shell agent.
//!
//! Runs a socket listener (Unix domain socket on macOS/Linux, named pipe on
//! Windows) that the `bypass-shell` client queries from a prompt hook. The
//! protocol is line-based and request/response per connection:
//!
//! - Client sends: `SYNC <gen>\n`
//! - Agent replies with either:
//!     - `UNCHANGED\n` when `<gen>` matches the current generation, or
//!     - `GEN <n>\n` followed by zero or more `VAR <KEY> <base64(value)>\n`
//!       lines and a final `END\n`.
//!
//! Values are base64-encoded so arbitrary bytes (including newlines) survive
//! the line-based framing. The client does the shell-specific quoting.

use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

use base64::Engine;
use interprocess::local_socket::traits::{Listener as _, Stream as _};
use interprocess::local_socket::{ListenerOptions, Stream};
use tauri::{AppHandle, Manager, State};

use crate::secrets::Vault;
use crate::state::AppState;

/// Handle to a running agent. Dropping does not stop it; call [`AgentHandle::stop`].
pub struct AgentHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl AgentHandle {
    /// Signals the accept loop to exit, wakes it with a throwaway connection,
    /// joins the thread, and removes the socket file.
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Ok(name) = socket_name() {
            // Unblock the blocking accept(); ignore any error.
            let _ = Stream::connect(name);
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        cleanup_socket_file();
    }
}

/// Bumps the generation counter so connected shells refresh on their next prompt.
pub fn bump_gen(state: &State<AppState>) {
    state.gen.fetch_add(1, Ordering::SeqCst);
}

/// Starts the listener on a background thread.
pub fn start(app: AppHandle) -> io::Result<AgentHandle> {
    #[cfg(unix)]
    {
        if let Some(p) = socket_path() {
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            // Remove a stale socket left by a previous crash so bind succeeds.
            let _ = std::fs::remove_file(&p);
        }
    }

    let listener = ListenerOptions::new().name(socket_name()?).create_sync()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Some(p) = socket_path() {
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600));
        }
    }

    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = stop.clone();
    let thread = std::thread::spawn(move || accept_loop(listener, stop_thread, app));

    Ok(AgentHandle {
        stop,
        thread: Some(thread),
    })
}

fn accept_loop(
    listener: interprocess::local_socket::Listener,
    stop: Arc<AtomicBool>,
    app: AppHandle,
) {
    loop {
        let conn = match listener.accept() {
            Ok(c) => c,
            Err(_) => {
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                continue;
            }
        };
        if stop.load(Ordering::SeqCst) {
            break;
        }
        let app = app.clone();
        // One short-lived thread per connection. Requests are sub-millisecond.
        std::thread::spawn(move || {
            let _ = handle_conn(conn, app);
        });
    }
}

fn handle_conn(mut conn: Stream, app: AppHandle) -> io::Result<()> {
    let request = read_line(&mut conn)?;
    let response = build_response(request.trim(), &app);
    conn.write_all(response.as_bytes())?;
    conn.flush()?;
    Ok(())
}

/// Reads a single `\n`-terminated line. The request is tiny so byte-at-a-time is fine.
fn read_line(conn: &mut impl Read) -> io::Result<String> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        let n = conn.read(&mut byte)?;
        if n == 0 || byte[0] == b'\n' {
            break;
        }
        buf.push(byte[0]);
    }
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn build_response(request: &str, app: &AppHandle) -> String {
    let client_gen = request
        .strip_prefix("SYNC ")
        .and_then(|s| s.trim().parse::<u64>().ok());

    let state = app.state::<AppState>();
    let current_gen = state.gen.load(Ordering::SeqCst);

    if client_gen == Some(current_gen) {
        return "UNCHANGED\n".to_string();
    }

    let active = state
        .config
        .lock()
        .ok()
        .and_then(|c| c.active_context.clone());

    let mut out = format!("GEN {current_gen}\n");
    if let Some(ctx) = active {
        if let Ok(pairs) = vars_for_context(&state, &ctx) {
            for (k, v) in pairs {
                let b64 = base64::engine::general_purpose::STANDARD.encode(v.as_bytes());
                out.push_str(&format!("VAR {k} {b64}\n"));
            }
        }
    }
    out.push_str("END\n");
    out
}

/// Decrypts every variable of `ctx`. Mirrors the lazy-vault pattern in
/// `credentials.rs`. The lock is released when this returns, before any socket I/O.
fn vars_for_context(state: &State<AppState>, ctx: &str) -> Result<Vec<(String, String)>, String> {
    let mut guard = state.vault.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(Vault::new().map_err(|e| e.to_string())?);
    }
    let vault = guard.as_ref().expect("vault initialized above");
    let keys = vault.list_keys(ctx).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(keys.len());
    for k in keys {
        let v = vault.get_var(ctx, &k).map_err(|e| e.to_string())?;
        out.push((k, v));
    }
    Ok(out)
}

// --- Socket addressing. Must match `crates/bypass-shell/src/socket.rs`. ------

#[cfg(unix)]
fn socket_path() -> Option<std::path::PathBuf> {
    Some(dirs::home_dir()?.join(".bypass").join("agent.sock"))
}

#[cfg(unix)]
fn socket_name() -> io::Result<interprocess::local_socket::Name<'static>> {
    use interprocess::local_socket::{GenericFilePath, ToFsName};
    let path =
        socket_path().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home dir"))?;
    path.to_fs_name::<GenericFilePath>()
}

#[cfg(unix)]
fn cleanup_socket_file() {
    if let Some(p) = socket_path() {
        let _ = std::fs::remove_file(p);
    }
}

#[cfg(windows)]
fn socket_name() -> io::Result<interprocess::local_socket::Name<'static>> {
    use interprocess::local_socket::{GenericNamespaced, ToNsName};
    "bypass-agent.sock".to_ns_name::<GenericNamespaced>()
}

#[cfg(windows)]
fn cleanup_socket_file() {}
