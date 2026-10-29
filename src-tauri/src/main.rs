// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ndi_ptz_lib::process_cleanup::pre_application_cleanup();
    let base_dir = ndi_ptz_lib::window_manager::get_app_base_dir();
    if std::env::var("WEBVIEW2_USER_DATA_FOLDER").is_err() {
        let webview_dir = base_dir.join(".data");
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &webview_dir);
    }
    ndi_ptz_lib::run();
}
