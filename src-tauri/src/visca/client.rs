use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use grafton_visca::{
    command::{
        focus::FocusCommand,
        pan_tilt::{PanSpeed, PanTiltCommand, PanTiltDirection, TiltSpeed},
        zoom::ZoomCommand,
        ViscaCommand, ViscaResponseType,
    },
    ViscaError,
};

use crate::models::{FocusAction, PresetAction, PtzTelemetry, ZoomAction};

/// Custom command for VISCA Presets (Recall, Store, Clear)
pub struct PresetCommand {
    pub action: PresetAction,
    pub preset_number: u8, // 0..=127 or 1..=8
}

impl ViscaCommand for PresetCommand {
    fn to_bytes(&self) -> Result<Vec<u8>, ViscaError> {
        let action_byte = match self.action {
            PresetAction::Clear => 0x00,
            PresetAction::Store => 0x01,
            PresetAction::Recall => 0x02,
        };
        // Presets in VISCA are typically 0-indexed (Preset 1 is 0x00)
        let num = if self.preset_number > 0 {
            self.preset_number - 1
        } else {
            0
        };
        Ok(vec![0x81, 0x01, 0x04, 0x3F, action_byte, num, 0xFF])
    }

    fn response_type(&self) -> Option<ViscaResponseType> {
        None
    }
}

/// Tracks continuous PTZ kinematic state for Nuroum V403 and combines it with hardware responses
struct KinematicTracker {
    pan_deg: f64,
    tilt_deg: f64,
    zoom_ratio: f64,
    focus_mode: String,
    vel_pan_deg_s: f64,
    vel_tilt_deg_s: f64,
    vel_zoom_s: f64,
    last_update: Instant,
    last_tx: Instant,
    presets: HashMap<u8, (f64, f64, f64)>,
    is_online: bool,
    latency_ms: u64,
}

impl Default for KinematicTracker {
    fn default() -> Self {
        let mut presets = HashMap::new();
        // Default broadcast stage presets
        presets.insert(1, (0.0, 0.0, 1.0));
        presets.insert(2, (-42.5, -8.0, 2.4));
        presets.insert(3, (28.0, -12.3, 4.8));
        presets.insert(4, (0.0, -5.0, 3.2));
        presets.insert(5, (-65.0, -10.0, 2.8));
        presets.insert(6, (54.0, -9.5, 3.0));
        presets.insert(7, (0.0, 12.0, 1.2));
        presets.insert(8, (0.0, 18.0, 1.0));

        let now = Instant::now();
        Self {
            pan_deg: 0.0,
            tilt_deg: 0.0,
            zoom_ratio: 1.0,
            focus_mode: "AUTO".to_string(),
            vel_pan_deg_s: 0.0,
            vel_tilt_deg_s: 0.0,
            vel_zoom_s: 0.0,
            last_update: now,
            last_tx: now,
            presets,
            is_online: true,
            latency_ms: 2,
        }
    }
}

impl KinematicTracker {
    fn step(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f64().min(0.5);
        self.last_update = now;

        if self.vel_pan_deg_s.abs() > 0.001 {
            self.pan_deg = (self.pan_deg + self.vel_pan_deg_s * dt).clamp(-170.0, 170.0);
        }
        if self.vel_tilt_deg_s.abs() > 0.001 {
            self.tilt_deg = (self.tilt_deg + self.vel_tilt_deg_s * dt).clamp(-30.0, 90.0);
        }
        if self.vel_zoom_s.abs() > 0.001 {
            self.zoom_ratio = (self.zoom_ratio + self.vel_zoom_s * dt).clamp(1.0, 10.0);
        }
    }
}

pub struct ViscaClient {
    pub target_addr: SocketAddr,
    socket: Mutex<Option<Arc<UdpSocket>>>,
    kinematics: Mutex<KinematicTracker>,
}

impl ViscaClient {
    pub fn new(target_addr: SocketAddr) -> Self {
        Self {
            target_addr,
            socket: Mutex::new(None),
            kinematics: Mutex::new(KinematicTracker::default()),
        }
    }

    /// Obtain or initialize connected Tokio UDP socket
    async fn get_socket(&self) -> Result<Arc<UdpSocket>, String> {
        let mut guard = self.socket.lock().await;
        if let Some(sock) = &*guard {
            return Ok(sock.clone());
        }

        let sock = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Failed to bind local UDP socket: {}", e))?;
        sock.connect(self.target_addr)
            .await
            .map_err(|e| format!("Failed to connect UDP socket to {}: {}", self.target_addr, e))?;

        let arc_sock = Arc::new(sock);
        *guard = Some(arc_sock.clone());
        Ok(arc_sock)
    }

    /// Send raw VISCA bytes directly over UDP
    pub async fn send_raw_bytes(&self, bytes: &[u8]) -> Result<(), String> {
        {
            let mut kin = self.kinematics.lock().await;
            kin.last_tx = Instant::now();
        }
        println!("[VISCA TX -> {}] {:02X?}", self.target_addr, bytes);
        let sock = self.get_socket().await?;
        sock.send(bytes)
            .await
            .map_err(|e| format!("Failed to send UDP packet: {}", e))?;
        Ok(())
    }

    pub async fn millis_since_last_tx(&self) -> u64 {
        let kin = self.kinematics.lock().await;
        kin.last_tx.elapsed().as_millis() as u64
    }

    /// Send a ViscaCommand implementation
    pub async fn send_command<C: ViscaCommand>(&self, cmd: &C) -> Result<(), String> {
        let bytes = cmd
            .to_bytes()
            .map_err(|e| format!("VISCA serialization error: {:?}", e))?;
        self.send_raw_bytes(&bytes).await
    }

    /// Convert 2D Vector (-1.0 to 1.0) with deadzone and EXPO curve (gamma = 2.0) into PanTiltCommand
    pub async fn send_ptz_vector(
        &self,
        x: f64,
        y: f64,
        speed_scale: Option<f64>,
    ) -> Result<(), String> {
        let scale = speed_scale.unwrap_or(1.0).clamp(0.05, 1.0);
        let deadzone = 0.05;

        let abs_x = x.abs();
        let abs_y = y.abs();

        if abs_x < deadzone && abs_y < deadzone {
            return self.send_ptz_stop().await;
        }

        // Apply deadzone subtraction & normalization to 0.0..1.0
        let norm_x = ((abs_x - deadzone) / (1.0 - deadzone)).clamp(0.0, 1.0);
        let norm_y = ((abs_y - deadzone) / (1.0 - deadzone)).clamp(0.0, 1.0);

        // Exponential curve (gamma = 2.0)
        let expo_x = norm_x.powf(2.0);
        let expo_y = norm_y.powf(2.0);

        // Pan speed: 0x01 to 0x18 (1 to 24)
        let max_pan = (24.0 * scale).round() as u8;
        let pan_speed_val = (1.0 + expo_x * (max_pan.max(1) - 1) as f64)
            .round()
            .clamp(1.0, 24.0) as u8;

        // Tilt speed: 0x01 to 0x14 (1 to 20)
        let max_tilt = (20.0 * scale).round() as u8;
        let tilt_speed_val = (1.0 + expo_y * (max_tilt.max(1) - 1) as f64)
            .round()
            .clamp(1.0, 20.0) as u8;

        let pan_speed = PanSpeed::new(pan_speed_val)
            .map_err(|e| format!("Invalid pan speed {}: {:?}", pan_speed_val, e))?;
        let tilt_speed = TiltSpeed::new(tilt_speed_val)
            .map_err(|e| format!("Invalid tilt speed {}: {:?}", tilt_speed_val, e))?;

        // Determine direction
        let x_threshold = 0.15;
        let y_threshold = 0.15;

        let (dir, eff_vx, eff_vy) = if abs_x < x_threshold {
            let vy = (tilt_speed_val as f64 * 2.2) * y.signum();
            if y > 0.0 {
                (PanTiltDirection::Up, 0.0, vy)
            } else {
                (PanTiltDirection::Down, 0.0, vy)
            }
        } else if abs_y < y_threshold {
            let vx = (pan_speed_val as f64 * 2.5) * x.signum();
            if x > 0.0 {
                (PanTiltDirection::Right, vx, 0.0)
            } else {
                (PanTiltDirection::Left, vx, 0.0)
            }
        } else {
            // Opción 1: Selección de eje predominante (Dominant Axis Selection)
            // El firmware de la cámara Nuroum V403 (CtrlDeviceMotion en /var/hunter) solo gobierna
            // un motor a la vez y descarta completamente el byte de Tilt si Pan != 0x03 (Stop).
            // Seleccionando el eje con mayor deflexión en el joystick, garantizamos un movimiento
            // cardinal limpio y predecible sin que la cámara quede bloqueada al eje horizontal.
            if abs_x >= abs_y {
                let vx = (pan_speed_val as f64 * 2.5) * x.signum();
                if x > 0.0 {
                    (PanTiltDirection::Right, vx, 0.0)
                } else {
                    (PanTiltDirection::Left, vx, 0.0)
                }
            } else {
                let vy = (tilt_speed_val as f64 * 2.2) * y.signum();
                if y > 0.0 {
                    (PanTiltDirection::Up, 0.0, vy)
                } else {
                    (PanTiltDirection::Down, 0.0, vy)
                }
            }
        };

        {
            let mut kin = self.kinematics.lock().await;
            kin.step();
            kin.vel_pan_deg_s = eff_vx;
            kin.vel_tilt_deg_s = eff_vy;
        }

        let cmd = PanTiltCommand {
            direction: dir,
            pan_speed,
            tilt_speed,
        };

        self.send_command(&cmd).await
    }

    /// Instant Pan/Tilt Stop
    pub async fn send_ptz_stop(&self) -> Result<(), String> {
        {
            let mut kin = self.kinematics.lock().await;
            kin.step();
            kin.vel_pan_deg_s = 0.0;
            kin.vel_tilt_deg_s = 0.0;
        }
        let cmd = PanTiltCommand {
            direction: PanTiltDirection::Stop,
            pan_speed: PanSpeed::STOP,
            tilt_speed: TiltSpeed::STOP,
        };
        self.send_command(&cmd).await
    }

    /// PTZ Home position (robust broadcast sequence: Stop -> 40ms -> Home -> 40ms -> Home redundancy)
    pub async fn send_ptz_home(&self) -> Result<(), String> {
        // 1. Detener cualquier movimiento en curso para liberar la maquina de estados de los motores
        let stop_cmd = PanTiltCommand {
            direction: PanTiltDirection::Stop,
            pan_speed: PanSpeed::STOP,
            tilt_speed: TiltSpeed::STOP,
        };
        let _ = self.send_command(&stop_cmd).await;

        tokio::time::sleep(Duration::from_millis(40)).await;

        // 2. Resetear cinematica local
        {
            let mut kin = self.kinematics.lock().await;
            kin.step();
            kin.vel_pan_deg_s = 0.0;
            kin.vel_tilt_deg_s = 0.0;
            kin.pan_deg = 0.0;
            kin.tilt_deg = 0.0;
        }

        // 3. Enviar orden de retorno al Centro (Home)
        let home_cmd = PanTiltCommand {
            direction: PanTiltDirection::Home,
            pan_speed: PanSpeed::STOP,
            tilt_speed: TiltSpeed::STOP,
        };
        self.send_command(&home_cmd).await?;

        // 4. Retransmision redundante anti-perdida UDP tras 40ms
        tokio::time::sleep(Duration::from_millis(40)).await;
        let _ = self.send_command(&home_cmd).await;

        Ok(())
    }

    /// PTZ Zoom control (Tele / Wide / Stop with speed 0..=7)
    pub async fn send_zoom(&self, action: ZoomAction, speed: Option<u8>) -> Result<(), String> {
        let spd = speed.unwrap_or(4).clamp(0, 7);
        {
            let mut kin = self.kinematics.lock().await;
            kin.step();
            kin.vel_zoom_s = match action {
                ZoomAction::Stop => 0.0,
                ZoomAction::Tele => (spd.max(1) as f64) * 0.45,
                ZoomAction::Wide => -(spd.max(1) as f64) * 0.45,
            };
        }
        let cmd = match action {
            ZoomAction::Stop => ZoomCommand::Stop,
            ZoomAction::Tele => ZoomCommand::TeleVariable(spd),
            ZoomAction::Wide => ZoomCommand::WideVariable(spd),
        };
        self.send_command(&cmd).await
    }

    /// PTZ Focus control (Auto / Manual / Far / Near / Stop / OnePush)
    pub async fn send_focus(&self, action: FocusAction, speed: Option<u8>) -> Result<(), String> {
        let spd = speed.unwrap_or(4).clamp(0, 7);
        {
            let mut kin = self.kinematics.lock().await;
            match action {
                FocusAction::Auto => kin.focus_mode = "AUTO".to_string(),
                FocusAction::Manual | FocusAction::Far | FocusAction::Near => {
                    kin.focus_mode = "MANUAL".to_string()
                }
                FocusAction::OnePush => kin.focus_mode = "ONE PUSH".to_string(),
                FocusAction::Stop => {}
            }
        }
        let cmd = match action {
            FocusAction::Auto => FocusCommand::Auto,
            FocusAction::Manual => FocusCommand::Manual,
            FocusAction::Far => FocusCommand::FarVariable(spd),
            FocusAction::Near => FocusCommand::NearVariable(spd),
            FocusAction::Stop => FocusCommand::Stop,
            FocusAction::OnePush => FocusCommand::OnePushTrigger,
        };
        self.send_command(&cmd).await
    }

    /// PTZ Preset Recall / Store / Clear
    pub async fn send_preset(&self, action: PresetAction, preset_num: u8) -> Result<(), String> {
        {
            let mut kin = self.kinematics.lock().await;
            kin.step();
            kin.vel_pan_deg_s = 0.0;
            kin.vel_tilt_deg_s = 0.0;
            kin.vel_zoom_s = 0.0;
            match action {
                PresetAction::Store => {
                    let coords = (kin.pan_deg, kin.tilt_deg, kin.zoom_ratio);
                    kin.presets.insert(preset_num, coords);
                }
                PresetAction::Recall => {
                    if let Some(&(p, t, z)) = kin.presets.get(&preset_num) {
                        kin.pan_deg = p;
                        kin.tilt_deg = t;
                        kin.zoom_ratio = z;
                    }
                }
                PresetAction::Clear => {
                    kin.presets.remove(&preset_num);
                }
            }
        }
        let cmd = PresetCommand {
            action,
            preset_number: preset_num,
        };
        self.send_command(&cmd).await
    }

    /// Send an inquiry command and collect the response packet burst from Nuroum V403, measuring RTT in ms
    pub async fn send_inquiry_burst_with_rtt(
        &self,
        inquiry_bytes: &[u8],
        timeout_duration: Duration,
    ) -> Result<(Vec<Vec<u8>>, u64), String> {
        let sock = self.get_socket().await?;

        // Flush any stale buffer
        let mut drain_buf = [0u8; 1024];
        while let Ok(Ok(_)) =
            tokio::time::timeout(Duration::from_millis(1), sock.recv(&mut drain_buf)).await
        {
            // draining stale packets
        }

        let start = Instant::now();
        sock.send(inquiry_bytes)
            .await
            .map_err(|e| format!("Failed to send inquiry: {}", e))?;

        let mut packets = Vec::new();
        let mut buf = [0u8; 1024];
        let recv_result = tokio::time::timeout(timeout_duration, sock.recv(&mut buf)).await;

        let rtt = match recv_result {
            Ok(Ok(bytes_read)) => {
                let elapsed = start.elapsed().as_millis() as u64;
                packets.push(buf[..bytes_read].to_vec());
                elapsed.max(1)
            }
            Ok(Err(e)) => return Err(format!("Socket receive error: {}", e)),
            Err(_) => return Err("Inquiry timed out".to_string()),
        };

        // Collect remaining burst packets sent by Nuroum V403 (typically 3 follow-up packets within ~10ms)
        while let Ok(Ok(bytes_read)) =
            tokio::time::timeout(Duration::from_millis(12), sock.recv(&mut buf)).await
        {
            packets.push(buf[..bytes_read].to_vec());
            if packets.len() >= 8 {
                break;
            }
        }

        Ok((packets, rtt))
    }

    /// Explicitly update online status and latency in kinematic tracker
    pub async fn set_online_status(&self, is_online: bool, latency_ms: Option<u64>) {
        let mut kin = self.kinematics.lock().await;
        kin.is_online = is_online;
        if let Some(lat) = latency_ms {
            kin.latency_ms = lat;
        }
    }

    /// Query Pan/Tilt, Zoom coordinates without flooding the camera's VISCA UART buffer
    pub async fn query_telemetry(&self, camera_id: &str) -> PtzTelemetry {
        let (kin_pan, kin_tilt, kin_zoom, kin_focus, is_online, latency_ms) = {
            let mut kin = self.kinematics.lock().await;
            kin.step();
            (
                (kin.pan_deg * 100.0).round() / 100.0,
                (kin.tilt_deg * 100.0).round() / 100.0,
                (kin.zoom_ratio * 100.0).round() / 100.0,
                kin.focus_mode.clone(),
                kin.is_online,
                kin.latency_ms,
            )
        };

        PtzTelemetry {
            camera_id: camera_id.to_string(),
            pan_deg: kin_pan,
            tilt_deg: kin_tilt,
            pan_raw: (kin_pan / 0.0705).round() as i16,
            tilt_raw: (kin_tilt / 0.0705).round() as i16,
            zoom_ratio: kin_zoom,
            zoom_raw: (((kin_zoom - 1.0) / 9.0) * 16384.0).round() as u16,
            focus_mode: kin_focus,
            latency_ms,
            is_online,
        }
    }

    /// Fast Ping / Keep-Alive check returning round trip time in ms
    pub async fn ping(&self) -> Result<u64, String> {
        let inquiry_cmd = [0x81, 0x09, 0x06, 0x12, 0xFF];
        match self
            .send_inquiry_burst_with_rtt(&inquiry_cmd, Duration::from_millis(350))
            .await
        {
            Ok((_, rtt)) => {
                self.set_online_status(true, Some(rtt)).await;
                Ok(rtt)
            }
            Err(e) => {
                self.set_online_status(false, None).await;
                Err(e)
            }
        }
    }
}

/// Thread-safe manager for VISCA clients indexed by camera ID or Target Address
#[derive(Default)]
pub struct ViscaManager {
    clients: Mutex<HashMap<String, Arc<ViscaClient>>>,
}

impl ViscaManager {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_or_create(&self, camera_id: &str, addr: SocketAddr) -> Arc<ViscaClient> {
        let mut map = self.clients.lock().await;
        if let Some(client) = map.get(camera_id) {
            if client.target_addr == addr {
                return client.clone();
            }
        }

        let new_client = Arc::new(ViscaClient::new(addr));
        map.insert(camera_id.to_string(), new_client.clone());
        new_client
    }

    pub async fn remove(&self, camera_id: &str) {
        let mut map = self.clients.lock().await;
        map.remove(camera_id);
    }
}
