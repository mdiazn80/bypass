use std::collections::BTreeMap;
use std::net::IpAddr;
use std::time::Duration;

use serde::Serialize;
use sysinfo::System;

/// Static-ish facts about the machine the app runs on.
#[derive(Debug, Clone, Serialize)]
pub struct MachineInfo {
    pub hostname: String,
    /// Friendly OS name, e.g. "macOS 15.6".
    pub os: String,
    pub kernel_version: String,
    pub arch: String,
    pub cpu_brand: String,
    pub cpu_count: usize,
    /// Bytes.
    pub total_memory: u64,
    /// Bytes.
    pub used_memory: u64,
    /// Seconds since boot.
    pub uptime: u64,
}

/// One network adapter with every address bound to it.
#[derive(Debug, Clone, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    /// True for the loopback adapter, which also carries a link-local address
    /// alongside 127.0.0.1 / ::1 on some platforms.
    pub is_loopback: bool,
}

/// The addresses the outside world sees. Either family may be missing when the
/// network has no connectivity for it, which is normal for IPv6.
#[derive(Debug, Clone, Serialize)]
pub struct PublicIps {
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
}

/// Everything the dashboard needs about the local machine. Cheap and offline:
/// public addresses are fetched separately so the view can render immediately.
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub machine: MachineInfo,
    pub interfaces: Vec<NetworkInterface>,
}

fn machine_info() -> MachineInfo {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu_list(sysinfo::CpuRefreshKind::nothing());

    // `long_os_version` is the marketing name ("macOS 26.6.2"); `name` alone is
    // the kernel's ("Darwin"), which is not what a user expects to read here.
    let os = System::long_os_version()
        .or_else(|| match (System::name(), System::os_version()) {
            (Some(name), Some(version)) => Some(format!("{name} {version}")),
            (Some(name), None) => Some(name),
            (None, Some(version)) => Some(version),
            (None, None) => None,
        })
        .unwrap_or_else(|| std::env::consts::OS.to_string());

    MachineInfo {
        hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
        os,
        kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
        arch: System::cpu_arch(),
        cpu_brand: sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .filter(|b| !b.is_empty())
            .unwrap_or_else(|| "unknown".to_string()),
        cpu_count: sys.cpus().len(),
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        uptime: System::uptime(),
    }
}

/// Groups the flat `(interface, address)` list the OS gives us into one entry
/// per adapter, keeping IPv4 and IPv6 apart. Ordering is alphabetical by name
/// so the dashboard does not reshuffle between refreshes.
fn interfaces() -> Vec<NetworkInterface> {
    let addresses = match local_ip_address::list_afinet_netifas() {
        Ok(list) => list,
        Err(_) => Vec::new(),
    };

    let mut grouped: BTreeMap<String, (Vec<String>, Vec<String>)> = BTreeMap::new();
    for (name, addr) in addresses {
        let entry = grouped.entry(name).or_default();
        match addr {
            IpAddr::V4(v4) => entry.0.push(v4.to_string()),
            IpAddr::V6(v6) => entry.1.push(v6.to_string()),
        }
    }

    grouped
        .into_iter()
        .map(|(name, (ipv4, ipv6))| {
            let is_loopback = ipv4.iter().chain(ipv6.iter()).any(|a| {
                a.parse::<IpAddr>()
                    .map(|ip| ip.is_loopback())
                    .unwrap_or(false)
            });
            NetworkInterface {
                name,
                ipv4,
                ipv6,
                is_loopback,
            }
        })
        .collect()
}

/// Local machine facts plus every address bound to a network adapter.
#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        machine: machine_info(),
        interfaces: interfaces(),
    }
}

/// Endpoints are pinned to a single address family: `api4`/`ipv4` hostnames
/// only resolve to A records and `api6`/`ipv6` only to AAAA, so the request
/// itself cannot fall back to the other family and report the wrong address.
const IPV4_ENDPOINTS: [&str; 2] = ["https://api4.ipify.org", "https://ipv4.icanhazip.com"];
const IPV6_ENDPOINTS: [&str; 2] = ["https://api6.ipify.org", "https://ipv6.icanhazip.com"];

/// Tries each endpoint in order and returns the first address that parses.
/// Returns `None` rather than an error: no IPv6 connectivity is a normal
/// result, not a failure the user needs to see.
async fn fetch_public_ip(client: reqwest::Client, endpoints: &'static [&'static str]) -> Option<String> {
    for url in endpoints {
        let Ok(response) = client.get(*url).send().await else {
            continue;
        };
        if !response.status().is_success() {
            continue;
        }
        let Ok(body) = response.text().await else {
            continue;
        };
        let trimmed = body.trim();
        if trimmed.parse::<IpAddr>().is_ok() {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Public IPv4 and IPv6 addresses, looked up concurrently.
#[tauri::command]
pub async fn get_public_ips() -> Result<PublicIps, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .map_err(|e| e.to_string())?;

    // Both lookups run concurrently: a missing IPv6 route should not add its
    // timeout on top of the IPv4 round trip.
    let v4 = tauri::async_runtime::spawn(fetch_public_ip(client.clone(), &IPV4_ENDPOINTS));
    let v6 = tauri::async_runtime::spawn(fetch_public_ip(client, &IPV6_ENDPOINTS));

    Ok(PublicIps {
        ipv4: v4.await.unwrap_or(None),
        ipv6: v6.await.unwrap_or(None),
    })
}
