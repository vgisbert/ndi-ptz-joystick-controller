use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub const ZLM_SECRET: &str = "035c73f7-bb6b-4889-a715-d9eb2d1925cc";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraHealthInfo {
    pub is_online: bool,
    pub visca_latency_ms: Option<u64>,
    pub http_ok: bool,
    pub http_rtt_ms: Option<u64>,
    pub active_streams_count: usize,
    pub total_readers: usize,
    pub bitrate_mbps: f64,
    pub fps: u32,
    pub resolution: String,
    pub uptime_seconds: u64,
    pub cpu_load_pct: f64,
}

async fn raw_http_call(ip: &str, req_bytes: &[u8], timeout_ms: u64) -> Result<String, String> {
    let clean_ip = ip.trim();
    if clean_ip.is_empty() || clean_ip.starts_with("0.") || clean_ip.starts_with("127.") {
        return Err(format!("Dirección IP no válida: '{}'", clean_ip));
    }
    let target = format!("{}:8090", clean_ip);

    let mut stream = tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        TcpStream::connect(&target),
    )
    .await
    .map_err(|_| format!("Tiempo de espera agotado al conectar a {}", target))?
    .map_err(|e| format!("Error conectando a {}: {}", target, e))?;

    stream
        .write_all(req_bytes)
        .await
        .map_err(|e| format!("Error enviando solicitud HTTP: {}", e))?;

    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 1024];

    let read_res = tokio::time::timeout(Duration::from_millis(1000), async {
        loop {
            match stream.read(&mut tmp).await {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&tmp[..n]);
                    if buf.windows(4).any(|w| w == b"\r\n\r\n") && buf.len() > 30 {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    })
    .await;

    let _ = read_res;
    let text = String::from_utf8_lossy(&buf).to_string();
    Ok(text)
}

/// Executes a JSON-RPC call against POST /index/api/remote on port 8090
pub async fn json_rpc_call(ip: &str, body_json: &str, timeout_ms: u64) -> Result<String, String> {
    let clean_ip = ip.trim();
    let target = format!("{}:8090", clean_ip);
    let request = format!(
        "POST /index/api/remote HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        target,
        body_json.len(),
        body_json
    );

    raw_http_call(clean_ip, request.as_bytes(), timeout_ms).await
}

/// Executes a GET request against ZLMediaKit on port 8090 with secret authentication
pub async fn zlmediakit_get(ip: &str, path_and_query: &str, timeout_ms: u64) -> Result<String, String> {
    let clean_ip = ip.trim();
    let target = format!("{}:8090", clean_ip);
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path_and_query,
        target
    );

    raw_http_call(clean_ip, request.as_bytes(), timeout_ms).await
}

/// Reboots the camera by triggering DevicePwr.Reboot JSON-RPC and ZLMediaKit restartServer
pub async fn reboot_camera(ip: &str) -> Result<bool, String> {
    let clean_ip = ip.trim();
    if clean_ip.is_empty() {
        return Err("Dirección IP vacía".to_string());
    }

    // 1. Invocamos DevicePwr.Reboot en /index/api/remote
    let payload = r#"{"Seq":1,"Class":"DevicePwr","Method":"Reboot","Param":{}}"#;
    let rpc_res = json_rpc_call(clean_ip, payload, 1000).await;
    println!("[REBOOT] Llamada DevicePwr.Reboot en {}: {:?}", clean_ip, rpc_res);

    // 2. Invocamos restartServer con el secreto de fábrica ZLMediaKit (puerto 8090)
    let restart_path = format!("/index/api/restartServer?secret={}", ZLM_SECRET);
    let zlm_res = zlmediakit_get(clean_ip, &restart_path, 1500).await;
    println!("[REBOOT] Llamada restartServer en {}: {:?}", clean_ip, zlm_res);

    if rpc_res.is_ok() || zlm_res.is_ok() {
        Ok(true)
    } else {
        Err(format!("Error en reinicio de cámara en {}: {:?}", clean_ip, zlm_res.err().or(rpc_res.err())))
    }
}

/// Queries internal camera health, CPU load, bitrate, FPS and live streams via ZLMediaKit
pub async fn get_camera_health(ip: &str) -> Result<CameraHealthInfo, String> {
    let clean_ip = ip.trim();
    if clean_ip.is_empty() {
        return Err("Dirección IP vacía".to_string());
    }

    let start = std::time::Instant::now();
    let threads_path = format!("/index/api/getThreadsLoad?secret={}", ZLM_SECRET);
    let threads_res = zlmediakit_get(clean_ip, &threads_path, 1200).await;
    let rtt_ms = start.elapsed().as_millis() as u64;

    let (http_ok, cpu_load) = match threads_res {
        Ok(raw) => {
            let json_str = if let Some(idx) = raw.find('{') { &raw[idx..] } else { "{}" };
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                let loads: Vec<f64> = v.pointer("/data")
                    .and_then(|d| d.as_array())
                    .map(|arr| arr.iter().filter_map(|item| item.get("load").and_then(|l| l.as_f64())).collect())
                    .unwrap_or_default();
                let avg = if !loads.is_empty() {
                    loads.iter().sum::<f64>() / loads.len() as f64
                } else {
                    0.0
                };
                (true, avg)
            } else {
                (true, 0.0)
            }
        }
        Err(_) => (false, 0.0),
    };

    let media_path = format!("/index/api/getMediaList?secret={}", ZLM_SECRET);
    let media_res = zlmediakit_get(clean_ip, &media_path, 1200).await;
    let mut streams_count = 0;
    let mut total_readers = 0;
    let mut total_bitrate_mbps = 0.0;
    let mut fps = 25u32;
    let mut resolution = "1080p (DSP)".to_string();
    let mut uptime = 0u64;

    if let Ok(raw) = media_res {
        let json_str = if let Some(idx) = raw.find('{') { &raw[idx..] } else { "{}" };
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
            if let Some(arr) = v.pointer("/data").and_then(|d| d.as_array()) {
                streams_count = arr.len();
                for stream in arr {
                    let readers = stream.get("readerCount").and_then(|r| r.as_u64()).unwrap_or(0) as usize;
                    total_readers += readers;
                    let speed = stream.get("bytesSpeed").and_then(|b| b.as_f64()).unwrap_or(0.0);
                    total_bitrate_mbps += (speed * 8.0) / 1_000_000.0;
                    if let Some(alive) = stream.get("aliveSecond").and_then(|a| a.as_u64()) {
                        uptime = uptime.max(alive);
                    }
                    if let Some(tracks) = stream.get("tracks").and_then(|t| t.as_array()) {
                        for track in tracks {
                            if track.get("codec_type").and_then(|ct| ct.as_i64()).unwrap_or(-1) == 0 {
                                if let Some(f) = track.get("fps").and_then(|f| f.as_u64()) {
                                    // Según documentación Nuroum V403 Sección 7.4:
                                    // En modo anti-parpadeo a 50 Hz (PAL, frequency = 1 en config.ini),
                                    // la captura real es a 25.0 FPS pero el firmware hardcodea 30 FPS en cabeceras NDI.
                                    fps = if f == 30 || f == 25 || f == 0 { 25 } else { f as u32 };
                                }
                                let w = track.get("width").and_then(|w| w.as_u64()).unwrap_or(1920);
                                let h = track.get("height").and_then(|h| h.as_u64()).unwrap_or(1080);
                                resolution = format!("{}x{}", w, h);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(CameraHealthInfo {
        is_online: http_ok,
        visca_latency_ms: None,
        http_ok,
        http_rtt_ms: if http_ok { Some(rtt_ms) } else { None },
        active_streams_count: streams_count,
        total_readers,
        bitrate_mbps: (total_bitrate_mbps * 10.0).round() / 10.0,
        fps,
        resolution,
        uptime_seconds: uptime,
        cpu_load_pct: (cpu_load * 10.0).round() / 10.0,
    })
}
