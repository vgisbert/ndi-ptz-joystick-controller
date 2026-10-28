// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ndi_ptz_lib::process_cleanup::pre_application_cleanup();
    ndi_ptz_lib::run();
}
