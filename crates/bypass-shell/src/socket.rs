//! Shared socket addressing. Must agree byte-for-byte with the agent side in
//! `src-tauri/src/agent.rs`.
//!
//! - Unix (macOS/Linux): filesystem socket at `~/.bypass/agent.sock`.
//! - Windows: a named pipe in the local namespace.

use interprocess::local_socket::Name;
use std::io;

#[cfg(windows)]
pub fn name() -> io::Result<Name<'static>> {
    use interprocess::local_socket::{GenericNamespaced, ToNsName};
    "bypass-agent.sock".to_ns_name::<GenericNamespaced>()
}

#[cfg(unix)]
pub fn name() -> io::Result<Name<'static>> {
    use interprocess::local_socket::{GenericFilePath, ToFsName};
    let path = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home dir"))?
        .join(".bypass")
        .join("agent.sock");
    path.to_fs_name::<GenericFilePath>()
}
