use std::fs;
use std::time::{Duration, SystemTime};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::hosts;
use crate::state::AppState;

/// How often the hosts file's metadata is checked. Cheap: one `stat` call.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Sent as the `hosts-changed` event whenever the hosts file is written, by
/// Bypass or by anything else (a VPN client, Docker, a script, an OS update).
#[derive(Clone, Serialize)]
struct HostsChanged {
    /// False when the managed block no longer matches the enabled contexts.
    in_sync: bool,
}

/// Size and modification time; a change in either means the file was written.
fn fingerprint() -> Option<(u64, SystemTime)> {
    let meta = fs::metadata(hosts::hosts_path()).ok()?;
    Some((meta.len(), meta.modified().ok()?))
}

/// Watches the system hosts file in a background thread for the lifetime of
/// the app. Polling avoids a file-watcher dependency, and `/etc/hosts` is
/// replaced rather than edited in place by many tools, which trips up
/// event-based watchers anyway.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last = fingerprint();
        loop {
            std::thread::sleep(POLL_INTERVAL);
            let current = fingerprint();
            if current == last {
                continue;
            }
            last = current;

            let Ok(content) = hosts::read_system_hosts() else {
                continue;
            };
            let in_sync = {
                let state = app.state::<AppState>();
                let Ok(contexts) = state.contexts.lock() else {
                    continue;
                };
                hosts::is_in_sync(&content, &contexts)
            };
            app.emit("hosts-changed", HostsChanged { in_sync }).ok();
        }
    });
}
