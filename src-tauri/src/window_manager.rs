/// Returns the canonical directory for storing all persistent application data:
/// - Development mode (cargo run / arrancar.bat): directory of arrancar.bat
/// - Release / Portable mode: directory of the current executable (.exe)
pub fn get_app_base_dir() -> PathBuf {
    // 1. Check CARGO_MANIFEST_DIR (active during cargo run / dev mode)
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(manifest);
        if let Some(parent) = p.parent() {
            if parent.join("arrancar.bat").exists() {
                return parent.to_path_buf();
            }
        }
    }

    // 2. Check current executable path
    if let Ok(exe_path) = std::env::current_exe() {
        let s = exe_path.to_string_lossy().to_lowercase();
        if s.contains("target\\debug") || s.contains("target/debug") {
            let mut cur = exe_path.clone();
            while let Some(parent) = cur.parent() {
                if parent.join("arrancar.bat").exists() {
                    return parent.to_path_buf();
                }
                cur = parent.to_path_buf();
            }
        }

        // Portable release mode: directory of the running .exe
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.to_path_buf();
        }
    }

    // 3. Fallback: check current directory or parent for arrancar.bat
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("arrancar.bat").exists() {
            return cwd;
        }
        if let Some(parent) = cwd.parent() {
            if parent.join("arrancar.bat").exists() {
                return parent.to_path_buf();
            }
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub always_on_top: bool,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 460,
            height: 740,
            always_on_top: false,
        }
    }
}

pub struct WindowManager {
    config_path: PathBuf,
    states: RwLock<HashMap<String, WindowGeometry>>,
}

impl WindowManager {
    pub fn new() -> Self {
        let base_dir = get_app_base_dir();
        let _ = fs::create_dir_all(&base_dir);
        let config_path = base_dir.join("window_states.json");
        let states = if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

        Self {
            config_path,
            states: RwLock::new(states),
        }
    }

    pub fn get_state(&self, label: &str) -> WindowGeometry {
        let map = self.states.read().unwrap();
        map.get(label).cloned().unwrap_or_default()
    }

    pub fn save_state(&self, label: &str, geometry: WindowGeometry) {
        {
            let mut map = self.states.write().unwrap();
            map.insert(label.to_string(), geometry);
        }
        self.persist_to_disk();
    }

    fn persist_to_disk(&self) {
        let map = self.states.read().unwrap();
        if let Ok(json) = serde_json::to_string_pretty(&*map) {
            let _ = fs::write(&self.config_path, json);
        }
    }
}

/// Helper to sanitize label for Tauri window
pub fn sanitize_window_label(camera_id: &str) -> String {
    format!("win-{}", camera_id.replace(['.', ':', ' ', '/'], "-"))
}

/// Open an existing floating window or create a new one with restored multimonitor coordinates
pub fn open_or_focus_camera_window(
    app: &AppHandle,
    window_manager: &Arc<WindowManager>,
    camera_id: &str,
    camera_name: &str,
) -> Result<(), String> {
    let label = sanitize_window_label(camera_id);

    // If window already exists, bring it to front and focus
    if let Some(existing_window) = app.get_webview_window(&label) {
        let _ = existing_window.show();
        let _ = existing_window.unminimize();
        let _ = existing_window.set_focus();
        return Ok(());
    }

    // Retrieve previous window position/size or use default tactical dimensions
    let geom = window_manager.get_state(&label);

    let url_str = format!("floating.html?cam={}", camera_id);
    let title = format!("NDI PTZ: {}", camera_name);

    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url_str.into()))
        .title(&title)
        .inner_size(geom.width as f64, geom.height as f64)
        .min_inner_size(380.0, 560.0)
        .position(geom.x as f64, geom.y as f64)
        .resizable(true)
        .always_on_top(geom.always_on_top)
        .decorations(true)
        .build()
        .map_err(|e| format!("Failed to create floating window: {}", e))?;

    let _ = window.show();
    let _ = window.set_focus();

    Ok(())
}

/// Toggle Always-on-Top for a window and persist state
pub fn toggle_always_on_top(
    window: &WebviewWindow,
    window_manager: &Arc<WindowManager>,
) -> Result<bool, String> {
    let label = window.label();
    let mut geom = window_manager.get_state(label);
    geom.always_on_top = !geom.always_on_top;

    window
        .set_always_on_top(geom.always_on_top)
        .map_err(|e| format!("Failed to toggle always-on-top: {}", e))?;

    // Also update current position and size
    if let Ok(pos) = window.outer_position() {
        geom.x = pos.x;
        geom.y = pos.y;
    }
    if let Ok(size) = window.inner_size() {
        geom.width = size.width;
        geom.height = size.height;
    }

    window_manager.save_state(label, geom.clone());
    Ok(geom.always_on_top)
}

/// Save current geometry of a window
pub fn save_window_geometry(
    window: &WebviewWindow,
    window_manager: &Arc<WindowManager>,
) {
    let label = window.label();
    let mut geom = window_manager.get_state(label);

    if let Ok(pos) = window.outer_position() {
        geom.x = pos.x;
        geom.y = pos.y;
    }
    if let Ok(size) = window.inner_size() {
        geom.width = size.width;
        geom.height = size.height;
    }

    window_manager.save_state(label, geom);
}
