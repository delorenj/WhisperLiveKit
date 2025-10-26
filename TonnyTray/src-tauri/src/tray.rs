// System tray functionality - Requires migration to tauri-plugin-tray for v2
// TODO: Migrate to tauri-plugin-tray (https://v2.tauri.app/plugin/tray/)

use log::warn;
use tauri::AppHandle;

use crate::state::{SharedState, TrayState};

/// Build the system tray menu (STUB - requires tauri-plugin-tray for v2)
pub fn build_tray_menu() -> () {
    warn!("System tray requires tauri-plugin-tray for Tauri v2 - currently disabled");
    ()
}

/// Handle system tray events (STUB - requires tauri-plugin-tray for v2)
pub fn handle_tray_event(_app: &AppHandle, _event: ()) {
    warn!("System tray requires tauri-plugin-tray for Tauri v2 - currently disabled");
}

/// Update tray menu based on app state (STUB - requires tauri-plugin-tray for v2)
pub fn update_tray_menu(_app: &AppHandle, _state: &SharedState) {
    warn!("System tray requires tauri-plugin-tray for Tauri v2 - currently disabled");
}
