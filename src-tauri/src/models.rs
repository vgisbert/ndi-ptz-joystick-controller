use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub id: String,
    pub mac: String,
    pub name: String,
    pub ip: String,
    pub visca_port: u16,
    pub ndi_port: Option<u16>,
    pub is_manual: bool,
    pub is_online: bool,
    pub latency_ms: Option<u64>,
    pub last_seen_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualCameraInput {
    pub ip: String,
    pub visca_port: Option<u16>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtzTelemetry {
    pub camera_id: String,
    pub pan_deg: f64,
    pub tilt_deg: f64,
    pub pan_raw: i16,
    pub tilt_raw: i16,
    pub zoom_ratio: f64,
    pub zoom_raw: u16,
    pub focus_mode: String,
    pub latency_ms: u64,
    pub is_online: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PresetAction {
    Recall,
    Store,
    Clear,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ZoomAction {
    Tele,
    Wide,
    Stop,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FocusAction {
    Auto,
    Manual,
    Far,
    Near,
    Stop,
    OnePush,
}
