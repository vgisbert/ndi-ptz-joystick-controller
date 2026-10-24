pub mod commands;
pub mod discovery;
pub mod models;
pub mod tray;
pub mod visca;
pub mod window_manager;

use std::sync::{Arc, Mutex};
use tauri::Manager;
use commands::AppState;
use discovery::{CameraRegistry, MdnsScanner};
use visca::ViscaManager;
use window_manager::WindowManager;

pub fn run() {
    let registry = Arc::new(CameraRegistry::new());
    let visca_manager = Arc::new(ViscaManager::new());
    let window_manager = Arc::new(WindowManager::new());

    let state = AppState {
        registry: registry.clone(),
        visca_manager: visca_manager.clone(),
        window_manager,
    };

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            println!("NDI PTZ Joystick Controller (Tauri Core) inicializado.");

            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.show();
                let _ = main_win.unminimize();
                let _ = main_win.set_focus();
                println!("[OK] Ventana principal (main) desplegada y enfocada.");
            }

            // Initialize System Tray
            if let Err(e) = tray::setup_tray(app.handle()) {
                eprintln!("[WARN] No se pudo inicializar la bandeja del sistema: {}", e);
            } else {
                println!("[OK] Bandeja del sistema (System Tray) configurada.");
            }

            // Launch mDNS Discovery Scanner Daemon on UDP 5353 and keep it alive in app state
            let mut scanner = MdnsScanner::new(registry.clone(), Some(app.handle().clone()));
            if let Err(e) = scanner.start() {
                eprintln!("[WARN] No se pudo iniciar el escáner mDNS: {}", e);
            } else {
                println!("[OK] Demonio mDNS de autodescubrimiento iniciado en UDP 5353.");
            }
            app.manage(Mutex::new(scanner));

            // Launch dynamic VISCA UDP 1259 hot-plug & MAC discovery watcher
            discovery::start_visca_subnet_watcher(
                registry.clone(),
                visca_manager.clone(),
                app.handle().clone(),
            );

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_cameras,
            commands::scan_all_cameras,
            commands::add_camera,
            commands::update_camera_config,
            commands::scan_camera_ip,
            commands::remove_camera,
            commands::ptz_vector,
            commands::ptz_stop,
            commands::ptz_home,
            commands::ptz_zoom,
            commands::ptz_focus,
            commands::ptz_preset,
            commands::ptz_inquiry_telemetry,
            commands::ptz_ping,
            commands::send_raw_visca,
            commands::open_camera_window,
            commands::toggle_window_pin,
            commands::is_window_pinned,
            commands::minimize_window,
            commands::close_window,
            commands::save_window_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
