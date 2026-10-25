use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State, WebviewWindow};
use tokio::net::UdpSocket;

use crate::discovery::{perform_network_discovery_cycle, probe_single_camera_ip, CameraRegistry};
use crate::models::{
    CameraInfo, FocusAction, PresetAction, PtzTelemetry, ZoomAction,
};
use crate::visca::{ViscaClient, ViscaManager};
use crate::window_manager::{
    open_or_focus_camera_window, save_window_geometry, toggle_always_on_top, WindowManager,
};

pub struct AppState {
    pub registry: Arc<CameraRegistry>,
    pub visca_manager: Arc<ViscaManager>,
    pub window_manager: Arc<WindowManager>,
}

async fn resolve_client(
    state: &State<'_, AppState>,
    camera_id: &str,
) -> Result<Arc<ViscaClient>, String> {
    let cam = state
        .registry
        .get(camera_id)
        .ok_or_else(|| format!("Camera ID '{}' not found in registry", camera_id))?;

    let ip: IpAddr = cam
        .ip
        .parse()
        .map_err(|e| format!("Invalid IP address '{}': {}", cam.ip, e))?;

    let addr = SocketAddr::new(ip, cam.visca_port);
    Ok(state.visca_manager.get_or_create(camera_id, addr).await)
}

#[tauri::command]
pub fn get_cameras(state: State<'_, AppState>) -> Vec<CameraInfo> {
    state.registry.list_all()
}

#[tauri::command]
pub async fn scan_all_cameras(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<CameraInfo>, String> {
    let list = perform_network_discovery_cycle(
        &state.registry,
        &state.visca_manager,
        Some(&app_handle),
        true,
    )
    .await;
    Ok(list)
}

#[tauri::command]
pub async fn add_camera(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    ip: String,
    port: Option<u16>,
    name: Option<String>,
) -> Result<CameraInfo, String> {
    let visca_port = port.unwrap_or(1259);
    let ipv4: Ipv4Addr = ip
        .trim()
        .parse()
        .map_err(|e| format!("Dirección IP inválida: {}", e))?;

    let rtt = probe_single_camera_ip(ipv4, visca_port, 450)
        .await
        .ok_or_else(|| format!("Sin respuesta VISCA en {}:{}", ipv4, visca_port))?;

    let (cam, _) = state.registry.upsert_discovered_by_ip(
        ipv4.to_string(),
        visca_port,
        Some(rtt),
        name,
        None,
    );
    let client = state
        .visca_manager
        .get_or_create(&cam.id, SocketAddr::new(IpAddr::V4(ipv4), visca_port))
        .await;
    client.set_online_status(true, Some(rtt)).await;

    let _ = app_handle.emit("cameras-updated", &state.registry.list_all());
    Ok(cam)
}

#[tauri::command]
pub async fn update_camera_config(
    state: State<'_, AppState>,
    camera_id: String,
    ip: String,
    port: Option<u16>,
    name: Option<String>,
) -> Result<CameraInfo, String> {
    let visca_port = port.unwrap_or(1259);
    let clean_ip = ip.trim().to_string();
    clean_ip
        .parse::<IpAddr>()
        .map_err(|e| format!("Dirección IP inválida '{}': {}", clean_ip, e))?;

    let cam = state
        .registry
        .update_camera_slot(&camera_id, clean_ip, visca_port, name, None, None);
    Ok(cam)
}

#[tauri::command]
pub async fn scan_camera_ip(
    state: State<'_, AppState>,
    camera_id: String,
    hint_ip: String,
    exclude_ip: String,
) -> Result<CameraInfo, String> {
    let visca_port: u16 = 1259;
    let excluded = exclude_ip.trim().to_string();
    let hinted = hint_ip.trim().to_string();

    // Collect /24 subnet prefixes [a, b, c] to scan
    let mut prefixes: Vec<[u8; 3]> = Vec::new();
    let mut push_prefix = |ip_str: &str| {
        if let Ok( ipv4 ) = ip_str.trim().parse::<Ipv4Addr>() {
            let oct = ipv4.octets();
            if oct[0] != 127 && oct[0] != 0 {
                let p = [oct[0], oct[1], oct[2]];
                if !prefixes.contains(&p) {
                    prefixes.push(p);
                }
            }
        }
    };

    push_prefix(&hinted);
    push_prefix(&excluded);

    // Also detect active local interface IPv4 /24 prefix
    if let Ok(tmp_sock) = std::net::UdpSocket::bind("0.0.0.0:0") {
        let probe_target = if !hinted.is_empty() {
            format!("{}:1259", hinted)
        } else {
            "8.8.8.8:80".to_string()
        };
        if tmp_sock.connect(&probe_target).is_ok() {
            if let Ok(local_addr) = tmp_sock.local_addr() {
                push_prefix(&local_addr.ip().to_string());
            }
        }
    }

    if prefixes.is_empty() {
        prefixes.push([192, 168, 9]);
    }

    let sock = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("No se pudo abrir socket UDP de búsqueda: {}", e))?;

    let inquiry_cmd: [u8; 5] = [0x81, 0x09, 0x06, 0x12, 0xFF];
    let start = Instant::now();

    // Priority targets: hinted IP and any mDNS-discovered cameras (excluding exclude_ip)
    let mut priority_ips: Vec<Ipv4Addr> = Vec::new();
    if let Ok(ip) = hinted.parse::<Ipv4Addr>() {
        if ip.to_string() != excluded {
            priority_ips.push(ip);
        }
    }
    for known in state.registry.list_all() {
        if let Ok(ip) = known.ip.parse::<Ipv4Addr>() {
            if ip.to_string() != excluded && !priority_ips.contains(&ip) {
                priority_ips.push(ip);
            }
        }
    }

    for ip in &priority_ips {
        let _ = sock
            .send_to(&inquiry_cmd, SocketAddr::new(IpAddr::V4(*ip), visca_port))
            .await;
    }

    // Check fast reply from priority IPs first (80ms window)
    let mut buf = [0u8; 256];
    let fast_deadline = Instant::now() + Duration::from_millis(80);
    while Instant::now() < fast_deadline {
        let rem = fast_deadline.saturating_duration_since(Instant::now());
        match tokio::time::timeout(rem, sock.recv_from(&mut buf)).await {
            Ok(Ok((n, src))) if n >= 3 => {
                let found_ip = src.ip().to_string();
                if found_ip != excluded && (buf[0] == 0x90 || buf[n - 1] == 0xFF) {
                    let rtt = start.elapsed().as_millis().max(1) as u64;
                    let cam = state.registry.update_camera_slot(
                        &camera_id,
                        found_ip,
                        visca_port,
                        None,
                        Some(true),
                        Some(rtt),
                    );
                    if let Ok(client) = resolve_client(&state, &camera_id).await {
                        client.set_online_status(true, Some(rtt)).await;
                    }
                    return Ok(cam);
                }
            }
            _ => break,
        }
    }

    // Full /24 sweep on UDP 1259 for any reassigned DHCP IP != exclude_ip
    let mut sent_ips = HashSet::new();
    for p in &prefixes {
        for host in 1u8..=254u8 {
            let ip = Ipv4Addr::new(p[0], p[1], p[2], host);
            let ip_str = ip.to_string();
            if ip_str == excluded || !sent_ips.insert(ip) {
                continue;
            }
            let _ = sock
                .send_to(&inquiry_cmd, SocketAddr::new(IpAddr::V4(ip), visca_port))
                .await;

            // Yield briefly every 32 packets and check if a response already arrived
            if host % 32 == 0 {
                while let Ok(Ok((n, src))) =
                    tokio::time::timeout(Duration::from_millis(2), sock.recv_from(&mut buf)).await
                {
                    if n >= 3 {
                        let found_ip = src.ip().to_string();
                        if found_ip != excluded && (buf[0] == 0x90 || buf[n - 1] == 0xFF) {
                            let rtt = start.elapsed().as_millis().max(1) as u64;
                            let cam = state.registry.update_camera_slot(
                                &camera_id,
                                found_ip,
                                visca_port,
                                None,
                                Some(true),
                                Some(rtt),
                            );
                            if let Ok(client) = resolve_client(&state, &camera_id).await {
                                client.set_online_status(true, Some(rtt)).await;
                            }
                            return Ok(cam);
                        }
                    }
                }
            }
        }
    }

    // Wait up to 650ms for any remaining responses from the subnet sweep
    let sweep_deadline = Instant::now() + Duration::from_millis(650);
    while Instant::now() < sweep_deadline {
        let rem = sweep_deadline.saturating_duration_since(Instant::now());
        match tokio::time::timeout(rem, sock.recv_from(&mut buf)).await {
            Ok(Ok((n, src))) if n >= 3 => {
                let found_ip = src.ip().to_string();
                if found_ip != excluded && (buf[0] == 0x90 || buf[n - 1] == 0xFF) {
                    let rtt = start.elapsed().as_millis().max(1) as u64;
                    let cam = state.registry.update_camera_slot(
                        &camera_id,
                        found_ip,
                        visca_port,
                        None,
                        Some(true),
                        Some(rtt),
                    );
                    if let Ok(client) = resolve_client(&state, &camera_id).await {
                        client.set_online_status(true, Some(rtt)).await;
                    }
                    return Ok(cam);
                }
            }
            Ok(Ok(_)) => continue,
            _ => break,
        }
    }

    // Not found -> mark camera as OFFLINE while keeping its last IP
    let fallback_ip = if !hinted.is_empty() {
        hinted
    } else {
        state
            .registry
            .get(&camera_id)
            .map(|c| c.ip)
            .unwrap_or_else(|| "192.168.1.101".to_string())
    };

    let cam = state.registry.update_camera_slot(
        &camera_id,
        fallback_ip,
        visca_port,
        None,
        Some(false),
        None,
    );
    if let Ok(client) = resolve_client(&state, &camera_id).await {
        client.set_online_status(false, None).await;
    }

    Ok(cam)
}

#[tauri::command]
pub async fn remove_camera(
    state: State<'_, AppState>,
    camera_id: String,
) -> Result<bool, String> {
    state.visca_manager.remove(&camera_id).await;
    Ok(state.registry.remove(&camera_id).is_some())
}

#[tauri::command]
pub async fn ptz_vector(
    state: State<'_, AppState>,
    camera_id: String,
    x: f64,
    y: f64,
    speed_scale: Option<f64>,
) -> Result<(), String> {
    let client = resolve_client(&state, &camera_id).await?;
    client.send_ptz_vector(x, y, speed_scale).await
}

#[tauri::command]
pub async fn ptz_stop(
    state: State<'_, AppState>,
    camera_id: String,
) -> Result<(), String> {
    let client = resolve_client(&state, &camera_id).await?;
    client.send_ptz_stop().await
}

#[tauri::command]
pub async fn ptz_home(
    state: State<'_, AppState>,
    camera_id: String,
) -> Result<(), String> {
    let client = resolve_client(&state, &camera_id).await?;
    client.send_ptz_home().await
}

#[tauri::command]
pub async fn ptz_zoom(
    state: State<'_, AppState>,
    camera_id: String,
    action: ZoomAction,
    speed: Option<u8>,
) -> Result<(), String> {
    let client = resolve_client(&state, &camera_id).await?;
    client.send_zoom(action, speed).await
}

#[tauri::command]
pub async fn ptz_focus(
    state: State<'_, AppState>,
    camera_id: String,
    action: FocusAction,
    speed: Option<u8>,
) -> Result<(), String> {
    let client = resolve_client(&state, &camera_id).await?;
    client.send_focus(action, speed).await
}

#[tauri::command]
pub async fn ptz_recalibrate_focus(
    state: State<'_, AppState>,
    camera_id: String,
) -> Result<bool, String> {
    let cam = state
        .registry
        .get(&camera_id)
        .ok_or_else(|| format!("Camera ID '{}' not found in registry", camera_id))?;

    // 1. Restaurar Focus Mode: Auto vía HTTP RPC en puerto 8090
    let _ = crate::discovery::http_set_focus_auto(&cam.ip).await;

    // 2. Si la cámara está disponible, forzar ciclo de nitidez con micro-zoom
    if let Ok(client) = resolve_client(&state, &camera_id).await {
        let _ = client.send_zoom(ZoomAction::Tele, Some(3)).await;
        tokio::time::sleep(Duration::from_millis(350)).await;
        let _ = client.send_zoom(ZoomAction::Wide, Some(3)).await;
        tokio::time::sleep(Duration::from_millis(350)).await;
        let _ = client.send_zoom(ZoomAction::Stop, None).await;
    }

    Ok(true)
}

#[tauri::command]
pub async fn ptz_preset(
    state: State<'_, AppState>,
    camera_id: String,
    action: PresetAction,
    preset_num: u8,
) -> Result<(), String> {
    let client = resolve_client(&state, &camera_id).await?;
    client.send_preset(action, preset_num).await
}

#[tauri::command]
pub async fn ptz_inquiry_telemetry(
    state: State<'_, AppState>,
    camera_id: String,
) -> Result<PtzTelemetry, String> {
    let client = resolve_client(&state, &camera_id).await?;
    let telemetry = client.query_telemetry(&camera_id).await;

    // Update status in registry
    state
        .registry
        .update_status(&camera_id, telemetry.is_online, Some(telemetry.latency_ms));

    Ok(telemetry)
}

#[tauri::command]
pub async fn ptz_ping(
    state: State<'_, AppState>,
    camera_id: String,
) -> Result<u64, String> {
    let client = resolve_client(&state, &camera_id).await?;
    match client.ping().await {
        Ok(rtt) => {
            state.registry.update_status(&camera_id, true, Some(rtt));
            Ok(rtt)
        }
        Err(e) => {
            state.registry.update_status(&camera_id, false, None);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn send_raw_visca(
    state: State<'_, AppState>,
    camera_id: String,
    hex_bytes: Vec<u8>,
) -> Result<(), String> {
    let client = resolve_client(&state, &camera_id).await?;
    client.send_raw_bytes(&hex_bytes).await
}

#[tauri::command]
pub async fn open_camera_window(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    camera_id: String,
) -> Result<(), String> {
    let camera_name = state
        .registry
        .get(&camera_id)
        .map(|c| c.name)
        .unwrap_or_else(|| camera_id.clone());

    open_or_focus_camera_window(&app_handle, &state.window_manager, &camera_id, &camera_name)
}

#[tauri::command]
pub fn toggle_window_pin(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    toggle_always_on_top(&window, &state.window_manager)
}

#[tauri::command]
pub fn is_window_pinned(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> bool {
    let geom = state.window_manager.get_state(window.label());
    geom.always_on_top
}

#[tauri::command]
pub fn minimize_window(window: WebviewWindow) -> Result<(), String> {
    window
        .minimize()
        .map_err(|e| format!("Failed to minimize: {}", e))
}

#[tauri::command]
pub fn close_window(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), String> {
    save_window_geometry(&window, &state.window_manager);
    window
        .close()
        .map_err(|e| format!("Failed to close: {}", e))
}

#[tauri::command]
pub fn save_window_state(
    window: WebviewWindow,
    state: State<'_, AppState>,
) {
    save_window_geometry(&window, &state.window_manager);
}

