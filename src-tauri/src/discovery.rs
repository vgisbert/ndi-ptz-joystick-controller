use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::windows::process::CommandExt;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mdns_sd::{ServiceDaemon, ServiceEvent};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::task::JoinSet;

use crate::models::CameraInfo;
use crate::visca::ViscaManager;

#[cfg(target_os = "windows")]
#[link(name = "iphlpapi")]
extern "system" {
    fn SendARP(dest_ip: u32, src_ip: u32, mac_addr: *mut u8, phy_addr_len: *mut u32) -> u32;
}

/// Resolves the hardware MAC address (XX:XX:XX:XX:XX:XX) of an IPv4 on the local LAN using Windows ARP table
pub fn resolve_mac_for_ip(ip_str: &str) -> String {
    if let Ok(ipv4) = ip_str.trim().parse::<Ipv4Addr>() {
        #[cfg(target_os = "windows")]
        unsafe {
            let dest_ip = u32::from_ne_bytes(ipv4.octets());
            let mut mac_buf = [0u8; 8];
            let mut phy_len: u32 = 6;
            let res = SendARP(dest_ip, 0, mac_buf.as_mut_ptr(), &mut phy_len);
            if res == 0 && phy_len >= 6 {
                let is_non_zero = mac_buf[..6].iter().any(|&b| b != 0);
                if is_non_zero {
                    return format!(
                        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        mac_buf[0], mac_buf[1], mac_buf[2], mac_buf[3], mac_buf[4], mac_buf[5]
                    );
                }
            }
        }
        let oct = ipv4.octets();
        return format!("02:00:{:02X}:{:02X}:{:02X}:{:02X}", oct[0], oct[1], oct[2], oct[3]);
    }
    "02:00:00:00:00:01".to_string()
}

pub fn mac_to_camera_id(mac: &str) -> String {
    let clean: String = mac
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    format!("cam-{}", clean)
}

fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as u64
}

/// Probes a single IPv4 on UDP 1259 using an isolated UdpSocket.
/// Sends up to 2 packets (the 1st triggers OS ARP resolution if the camera was just plugged in;
/// the 2nd delivers the VISCA inquiry once ARP is resolved).
pub async fn probe_single_camera_ip(ip: Ipv4Addr, port: u16, timeout_ms: u64) -> Option<u64> {
    let sock = UdpSocket::bind("0.0.0.0:0").await.ok()?;
    let target = SocketAddr::new(IpAddr::V4(ip), port);
    sock.connect(target).await.ok()?;

    let inquiry_cmd: [u8; 5] = [0x81, 0x09, 0x06, 0x12, 0xFF];
    let start = Instant::now();
    sock.send(&inquiry_cmd).await.ok()?;

    let mut buf = [0u8; 128];
    let first_wait = (timeout_ms / 3).max(45);
    match tokio::time::timeout(Duration::from_millis(first_wait), sock.recv(&mut buf)).await {
        Ok(Ok(n)) if n >= 3 && (buf[0] == 0x90 || buf[n - 1] == 0xFF) => {
            return Some(start.elapsed().as_millis().max(1) as u64);
        }
        _ => {}
    }

    // Second packet in case the first UDP datagram was dropped while Windows resolved ARP
    let _ = sock.send(&inquiry_cmd).await;
    let rem_wait = timeout_ms.saturating_sub(first_wait).max(65);
    match tokio::time::timeout(Duration::from_millis(rem_wait), sock.recv(&mut buf)).await {
        Ok(Ok(n)) if n >= 3 && (buf[0] == 0x90 || buf[n - 1] == 0xFF) => {
            Some(start.elapsed().as_millis().max(1) as u64)
        }
        _ => None,
    }
}

/// Silently ensures camera is in Focus Mode: Auto via HTTP RPC (port 8090)
pub async fn http_set_focus_auto(ip: &str) -> Result<(), String> {
    let clean_ip = ip.trim();
    if clean_ip.is_empty() || clean_ip.starts_with("0.") || clean_ip.starts_with("127.") {
        return Ok(());
    }
    let target = format!("{}:8090", clean_ip);
    let mut stream = match tokio::time::timeout(
        Duration::from_millis(800),
        TcpStream::connect(&target),
    )
    .await {
        Ok(Ok(s)) => s,
        _ => return Err(format!("No se pudo conectar a {}", target)),
    };

    let body = r#"{"Seq":1,"Class":"Focus","Method":"SetMode","Param":{"Mode":"Auto"}}"#;
    let request = format!(
        "POST /index/api/remote HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        target,
        body.len(),
        body
    );

    let _ = stream.write_all(request.as_bytes()).await;
    let mut buf = [0u8; 256];
    let _ = tokio::time::timeout(Duration::from_millis(400), stream.read(&mut buf)).await;
    println!("[AUTO-FOCUS] Modo Auto reasegurado por HTTP en {}", target);
    Ok(())
}

/// Reads current IPv4 hosts from the Windows ARP table without opening a console window
pub fn read_arp_table_candidates(subnets: &[[u8; 3]]) -> Vec<Ipv4Addr> {
    let mut priority = Vec::new();
    let mut others = Vec::new();

    if let Ok(output) = std::process::Command::new("arp")
        .arg("-a")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(ip) = parts[0].parse::<Ipv4Addr>() {
                    let oct = ip.octets();
                    if oct[3] == 0 || oct[3] == 255 || oct[0] == 224 || oct[0] == 239 {
                        continue;
                    }
                    let in_subnet = subnets
                        .iter()
                        .any(|s| s[0] == oct[0] && s[1] == oct[1] && s[2] == oct[2]);
                    if !in_subnet {
                        continue;
                    }
                    let mac_lower = parts[1].to_ascii_lowercase();
                    // Nuroum V403 OUI is 98-6e-e8
                    if mac_lower.starts_with("98-6e-e8") || mac_lower.starts_with("98:6e:e8") {
                        if !priority.contains(&ip) {
                            priority.push(ip);
                        }
                    } else if !others.contains(&ip) {
                        others.push(ip);
                    }
                }
            }
        }
    }

    priority.extend(others);
    priority
}

#[derive(Default)]
pub struct CameraRegistry {
    cameras: RwLock<HashMap<String, CameraInfo>>,
    order: RwLock<Vec<String>>,
    missed_pings: RwLock<HashMap<String, u8>>,
}

impl CameraRegistry {
    pub fn new() -> Self {
        Self {
            cameras: RwLock::new(HashMap::new()),
            order: RwLock::new(Vec::new()),
            missed_pings: RwLock::new(HashMap::new()),
        }
    }

    fn ensure_order(&self, id: &str) {
        let mut ord = self.order.write().unwrap();
        if !ord.iter().any(|x| x == id) {
            ord.push(id.to_string());
        }
    }

    pub fn upsert_discovered_by_ip(
        &self,
        ip: String,
        visca_port: u16,
        latency_ms: Option<u64>,
        name_hint: Option<String>,
        ndi_port: Option<u16>,
    ) -> (CameraInfo, bool) {
        let sanitized_ip = ip.trim().to_string();
        let mac = resolve_mac_for_ip(&sanitized_ip);
        let id = mac_to_camera_id(&mac);

        self.ensure_order(&id);
        {
            let mut mp = self.missed_pings.write().unwrap();
            mp.insert(id.clone(), 0);
        }

        let mut map = self.cameras.write().unwrap();

        let stale_ids: Vec<String> = map
            .iter()
            .filter(|(k, v)| *k != &id && v.ip == sanitized_ip)
            .map(|(k, _)| k.clone())
            .collect();
        for sid in stale_ids {
            map.remove(&sid);
        }

        if let Some(existing) = map.get_mut(&id) {
            let changed = !existing.is_online || existing.ip != sanitized_ip;
            existing.ip = sanitized_ip;
            existing.mac = mac;
            existing.visca_port = visca_port;
            existing.is_online = true;
            if let Some(lat) = latency_ms {
                existing.latency_ms = Some(lat);
            }
            if let Some(np) = ndi_port {
                existing.ndi_port = Some(np);
            }
            if let Some(nh) = name_hint {
                if !nh.trim().is_empty() && existing.is_manual {
                    existing.name = nh;
                }
            }
            existing.last_seen_epoch_ms = current_epoch_ms();
            (existing.clone(), changed)
        } else {
            let ord = self.order.read().unwrap();
            let idx = ord.iter().position(|x| x == &id).unwrap_or(map.len()) + 1;
            let default_name = name_hint
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| format!("CÁM {}", idx));

            let cam = CameraInfo {
                id: id.clone(),
                mac,
                name: default_name,
                ip: sanitized_ip.clone(),
                visca_port,
                ndi_port,
                is_manual: false,
                is_online: true,
                latency_ms,
                last_seen_epoch_ms: current_epoch_ms(),
            };
            map.insert(id, cam.clone());
            let ip_for_focus = sanitized_ip;
            tokio::spawn(async move {
                let _ = http_set_focus_auto(&ip_for_focus).await;
            });
            (cam, true)
        }
    }

    pub fn upsert_camera(&self, cam: CameraInfo) {
        self.ensure_order(&cam.id);
        let mut map = self.cameras.write().unwrap();
        if let Some(existing) = map.get_mut(&cam.id) {
            let was_offline = !existing.is_online;
            existing.ip = cam.ip.clone();
            existing.mac = cam.mac;
            existing.last_seen_epoch_ms = cam.last_seen_epoch_ms;
            existing.is_online = cam.is_online;
            if let Some(ndi_port) = cam.ndi_port {
                existing.ndi_port = Some(ndi_port);
            }
            if !existing.is_manual && !cam.name.is_empty() {
                existing.name = cam.name;
            }
            if was_offline && cam.is_online {
                let ip_for_focus = cam.ip.clone();
                tokio::spawn(async move {
                    let _ = http_set_focus_auto(&ip_for_focus).await;
                });
            }
        } else {
            if cam.is_online {
                let ip_for_focus = cam.ip.clone();
                tokio::spawn(async move {
                    let _ = http_set_focus_auto(&ip_for_focus).await;
                });
            }
            map.insert(cam.id.clone(), cam);
        }
    }

    pub fn add_manual(&self, ip: String, visca_port: u16, name: Option<String>) -> CameraInfo {
        let (cam, _) = self.upsert_discovered_by_ip(ip, visca_port, None, name, None);
        cam
    }

    pub fn update_camera_slot(
        &self,
        id: &str,
        ip: String,
        visca_port: u16,
        name: Option<String>,
        is_online: Option<bool>,
        latency_ms: Option<u64>,
    ) -> CameraInfo {
        let sanitized_ip = ip.trim().to_string();
        let mac = resolve_mac_for_ip(&sanitized_ip);
        self.ensure_order(id);

        let mut map = self.cameras.write().unwrap();
        if let Some(existing) = map.get_mut(id) {
            existing.ip = sanitized_ip;
            if existing.mac.is_empty() || existing.mac.starts_with("02:00:") {
                existing.mac = mac;
            }
            existing.visca_port = visca_port;
            if let Some(n) = name {
                if !n.trim().is_empty() {
                    existing.name = n;
                    existing.is_manual = true;
                }
            }
            if let Some(online) = is_online {
                existing.is_online = online;
                if online {
                    existing.last_seen_epoch_ms = current_epoch_ms();
                }
            }
            if latency_ms.is_some() {
                existing.latency_ms = latency_ms;
            }
            existing.clone()
        } else {
            let ord = self.order.read().unwrap();
            let idx = ord.iter().position(|x| x == id).unwrap_or(map.len()) + 1;
            let friendly_name = name.unwrap_or_else(|| format!("CÁM {}", idx));
            let cam = CameraInfo {
                id: id.to_string(),
                mac,
                name: friendly_name,
                ip: sanitized_ip,
                visca_port,
                ndi_port: None,
                is_manual: true,
                is_online: is_online.unwrap_or(true),
                latency_ms,
                last_seen_epoch_ms: current_epoch_ms(),
            };
            map.insert(id.to_string(), cam.clone());
            cam
        }
    }

    pub fn remove(&self, id: &str) -> Option<CameraInfo> {
        let mut map = self.cameras.write().unwrap();
        map.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<CameraInfo> {
        let map = self.cameras.read().unwrap();
        map.get(id).cloned()
    }

    /// Returns all currently online cameras ordered by discovery order
    pub fn list_all(&self) -> Vec<CameraInfo> {
        let map = self.cameras.read().unwrap();
        let ord = self.order.read().unwrap();
        let mut list: Vec<CameraInfo> = Vec::new();
        for id in ord.iter() {
            if let Some(cam) = map.get(id) {
                if cam.is_online {
                    list.push(cam.clone());
                }
            }
        }
        for (id, cam) in map.iter() {
            if cam.is_online && !ord.contains(id) {
                list.push(cam.clone());
            }
        }
        list
    }

    /// Returns all known cameras (including offline ones)
    pub fn list_known(&self) -> Vec<CameraInfo> {
        let map = self.cameras.read().unwrap();
        let ord = self.order.read().unwrap();
        let mut list: Vec<CameraInfo> = Vec::new();
        for id in ord.iter() {
            if let Some(cam) = map.get(id) {
                list.push(cam.clone());
            }
        }
        list
    }

    pub fn record_ping_result(&self, id: &str, success: bool, rtt: Option<u64>, immediate_offline: bool) -> bool {
        if success {
            {
                let mut mp = self.missed_pings.write().unwrap();
                mp.insert(id.to_string(), 0);
            }
            let mut map = self.cameras.write().unwrap();
            if let Some(cam) = map.get_mut(id) {
                let was_offline = !cam.is_online;
                cam.is_online = true;
                if let Some(r) = rtt {
                    cam.latency_ms = Some(r);
                }
                cam.last_seen_epoch_ms = current_epoch_ms();
                return was_offline;
            }
            false
        } else {
            let count = {
                let mut mp = self.missed_pings.write().unwrap();
                let entry = mp.entry(id.to_string()).or_insert(0);
                *entry = entry.saturating_add(1);
                *entry
            };
            let threshold = if immediate_offline { 1 } else { 2 };
            if count >= threshold {
                let mut map = self.cameras.write().unwrap();
                if let Some(cam) = map.get_mut(id) {
                    if cam.is_online {
                        cam.is_online = false;
                        return true;
                    }
                }
            }
            false
        }
    }

    pub fn update_status(&self, id: &str, is_online: bool, latency_ms: Option<u64>) {
        let mut map = self.cameras.write().unwrap();
        if let Some(cam) = map.get_mut(id) {
            cam.is_online = is_online;
            if let Some(lat) = latency_ms {
                cam.latency_ms = Some(lat);
            }
            if is_online {
                cam.last_seen_epoch_ms = current_epoch_ms();
            }
        }
    }
}

/// Detects the local /24 subnet prefixes active on the machine
pub fn detect_local_subnets(known_ips: &[String]) -> Vec<[u8; 3]> {
    let mut prefixes: Vec<[u8; 3]> = Vec::new();
    let mut push_ip = |ip_str: &str| {
        if let Ok(ipv4) = ip_str.trim().parse::<Ipv4Addr>() {
            let oct = ipv4.octets();
            if oct[0] != 127 && oct[0] != 0 && oct[0] != 169 {
                let p = [oct[0], oct[1], oct[2]];
                if !prefixes.contains(&p) {
                    prefixes.push(p);
                }
            }
        }
    };

    if let Ok(sock) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if sock.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = sock.local_addr() {
                push_ip(&local_addr.ip().to_string());
            }
        }
    }

    for ip in known_ips {
        push_ip(ip);
    }

    if prefixes.is_empty() {
        prefixes.push([192, 168, 9]);
    }
    prefixes
}

/// Runs a non-intrusive discovery cycle using isolated UDP sockets per IP:
/// 1. Checks already-online cameras (only when idle >2s so PTZ movement is never affected).
/// 2. Immediately probes ARP table entries (prioritizing Nuroum 98:6E:E8 MACs).
/// 3. Sweeps remaining IPs in the /24 subnet in parallel batches to catch newly plugged-in cameras.
pub async fn perform_network_discovery_cycle(
    registry: &Arc<CameraRegistry>,
    visca_manager: &Arc<ViscaManager>,
    app_handle: Option<&AppHandle>,
    force_full_scan: bool,
) -> Vec<CameraInfo> {
    let visca_port: u16 = 1259;
    let known = registry.list_known();
    let mut active_online_ips = HashSet::new();

    // 1. Verify existing online cameras (only if idle > 2000ms)
    for cam in &known {
        if cam.is_online {
            if let Ok(ip) = cam.ip.parse::<Ipv4Addr>() {
                let addr = SocketAddr::new(IpAddr::V4(ip), cam.visca_port);
                let client = visca_manager.get_or_create(&cam.id, addr).await;
                let idle_ms = client.millis_since_last_tx().await;

                if force_full_scan || idle_ms > 2000 {
                    if let Some(rtt) = probe_single_camera_ip(ip, cam.visca_port, 300).await {
                        registry.record_ping_result(&cam.id, true, Some(rtt), force_full_scan);
                        client.set_online_status(true, Some(rtt)).await;
                        active_online_ips.insert(ip);
                    } else {
                        registry.record_ping_result(&cam.id, false, None, force_full_scan);
                        if let Some(updated) = registry.get(&cam.id) {
                            if !updated.is_online {
                                client.set_online_status(false, None).await;
                            } else {
                                active_online_ips.insert(ip);
                            }
                        }
                    }
                } else {
                    // Actively moving with joystick -> keep online without probing
                    active_online_ips.insert(ip);
                }
            }
        }
    }

    // 2. Build candidate IPs to probe (known offline IPs + habitual DHCP IPs + ARP table + full /24 subnet)
    let known_ip_strings: Vec<String> = known.iter().map(|c| c.ip.clone()).collect();
    let prefixes = detect_local_subnets(&known_ip_strings);

    let mut fast_candidates: Vec<Ipv4Addr> = Vec::new();

    // Always include habitual Nuroum V403 DHCP addresses on the active subnet first (even if not in ARP cache yet)
    for p in &prefixes {
        for &host in &[171u8, 108u8] {
            let ip = Ipv4Addr::new(p[0], p[1], p[2], host);
            if !fast_candidates.contains(&ip) {
                fast_candidates.push(ip);
            }
        }
    }

    for cam in &known {
        if !cam.is_online {
            if let Ok(ip) = cam.ip.parse::<Ipv4Addr>() {
                if !fast_candidates.contains(&ip) {
                    fast_candidates.push(ip);
                }
            }
        }
    }

    for arp_ip in read_arp_table_candidates(&prefixes) {
        if !fast_candidates.contains(&arp_ip) {
            fast_candidates.push(arp_ip);
        }
    }

    // Remove IPs that are already verified online
    fast_candidates.retain(|ip| !active_online_ips.contains(ip));

    // Probe all priority / ARP candidates in parallel using isolated UdpSockets
    let mut set = JoinSet::new();
    for ip in fast_candidates.iter().copied() {
        set.spawn(async move {
            probe_single_camera_ip(ip, visca_port, 280)
                .await
                .map(|rtt| (ip, rtt))
        });
    }

    while let Some(res) = set.join_next().await {
        if let Ok(Some((ip, rtt))) = res {
            active_online_ips.insert(ip);
            let (cam, _) = registry.upsert_discovered_by_ip(
                ip.to_string(),
                visca_port,
                Some(rtt),
                None,
                None,
            );
            println!(
                "[DISCOVERY] Camara ONLINE: id={} mac={} ip={} ({} ms)",
                cam.id, cam.mac, cam.ip, rtt
            );
            let client = visca_manager
                .get_or_create(&cam.id, SocketAddr::new(IpAddr::V4(ip), visca_port))
                .await;
            client.set_online_status(true, Some(rtt)).await;
        }
    }

    // 3. Sweep remaining subnet IPs in batches of 24 (avoids Windows ARP queue saturation)
    let mut remaining_ips = Vec::new();
    for p in &prefixes {
        for host in 1u8..=254u8 {
            let ip = Ipv4Addr::new(p[0], p[1], p[2], host);
            if !active_online_ips.contains(&ip) && !fast_candidates.contains(&ip) {
                remaining_ips.push(ip);
            }
        }
    }

    for chunk in remaining_ips.chunks(24) {
        let mut chunk_set = JoinSet::new();
        for &ip in chunk {
            chunk_set.spawn(async move {
                probe_single_camera_ip(ip, visca_port, 140)
                    .await
                    .map(|rtt| (ip, rtt))
            });
        }
        while let Some(res) = chunk_set.join_next().await {
            if let Ok(Some((ip, rtt))) = res {
                let (cam, _) = registry.upsert_discovered_by_ip(
                    ip.to_string(),
                    visca_port,
                    Some(rtt),
                    None,
                    None,
                );
                println!(
                    "[DISCOVERY] Camara hallada en barrido: id={} mac={} ip={} ({} ms)",
                    cam.id, cam.mac, cam.ip, rtt
                );
                let client = visca_manager
                    .get_or_create(&cam.id, SocketAddr::new(IpAddr::V4(ip), visca_port))
                    .await;
                client.set_online_status(true, Some(rtt)).await;
            }
        }
    }

    let online_list = registry.list_all();
    if let Some(app) = app_handle {
        let _ = app.emit("cameras-updated", &online_list);
    }

    online_list
}

/// Spawns the continuous background hot-plug / unplug camera discovery watcher
pub fn start_visca_subnet_watcher(
    registry: Arc<CameraRegistry>,
    visca_manager: Arc<ViscaManager>,
    app_handle: AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        // Immediate discovery scan at launch
        perform_network_discovery_cycle(&registry, &visca_manager, Some(&app_handle), true).await;

        let mut interval = tokio::time::interval(Duration::from_secs(3));
        interval.tick().await;
        loop {
            interval.tick().await;
            perform_network_discovery_cycle(&registry, &visca_manager, Some(&app_handle), false)
                .await;
        }
    });
}

pub struct MdnsScanner {
    daemon: Option<ServiceDaemon>,
    registry: Arc<CameraRegistry>,
    app_handle: Option<AppHandle>,
}

impl MdnsScanner {
    pub fn new(registry: Arc<CameraRegistry>, app_handle: Option<AppHandle>) -> Self {
        Self {
            daemon: None,
            registry,
            app_handle,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.daemon.is_some() {
            return Ok(());
        }

        let daemon =
            ServiceDaemon::new().map_err(|e| format!("Failed to start mDNS daemon: {}", e))?;

        let service_types = [
            "_ndi._tcp.local.",
            "_ndi-hx._tcp.local.",
            "_visca._udp.local.",
        ];

        let registry = self.registry.clone();
        let app_handle = self.app_handle.clone();

        for service_type in service_types {
            match daemon.browse(service_type) {
                Ok(receiver) => {
                    let reg = registry.clone();
                    let app = app_handle.clone();
                    let st = service_type.to_string();

                    std::thread::spawn(move || {
                        while let Ok(event) = receiver.recv() {
                            match event {
                                ServiceEvent::ServiceResolved(info) => {
                                    let _fullname = info.get_fullname().to_string();
                                    let port = info.get_port();
                                    let addresses = info.get_addresses();

                                    let ip_opt = addresses
                                        .iter()
                                        .find(|a| matches!(a, IpAddr::V4(_)))
                                        .or_else(|| addresses.iter().next());

                                    if let Some(ip) = ip_opt {
                                        let ip_str = ip.to_string();
                                        let visca_port = if st.contains("_visca") {
                                            port
                                        } else {
                                            1259
                                        };
                                        let ndi_port = if st.contains("_ndi") {
                                            Some(port)
                                        } else {
                                            None
                                        };

                                        let (cam, _) = reg.upsert_discovered_by_ip(
                                            ip_str,
                                            visca_port,
                                            None,
                                            None,
                                            ndi_port,
                                        );

                                        if let Some(ref app) = app {
                                            let _ = app.emit("camera-discovered", &cam);
                                            let _ = app.emit("cameras-updated", &reg.list_all());
                                        }
                                    }
                                }
                                ServiceEvent::ServiceRemoved(_, fullname) => {
                                    println!("mDNS Service removed: {}", fullname);
                                }
                                _ => {}
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to browse {}: {}", service_type, e);
                }
            }
        }

        self.daemon = Some(daemon);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(daemon) = self.daemon.take() {
            let _ = daemon.shutdown();
        }
    }
}
