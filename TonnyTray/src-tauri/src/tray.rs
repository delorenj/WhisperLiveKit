use log::{debug, error};
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, SystemTraySubmenu,
};

use crate::state::{SharedState, TrayState};

/// Build the system tray menu
pub fn build_tray_menu() -> SystemTray {
    let start_recording = CustomMenuItem::new("start_recording", "🔴 Start Recording");
    let stop_recording = CustomMenuItem::new("stop_recording", "⏸️  Stop Recording").disabled();
    let separator1 = SystemTrayMenuItem::Separator;

    let status = CustomMenuItem::new("status", "📊 Status: Disconnected").disabled();
    let last_transcription = CustomMenuItem::new("last_transcription", "🔊 Last: ---").disabled();
    let separator2 = SystemTrayMenuItem::Separator;

    let settings = CustomMenuItem::new("settings", "⚙️  Settings");
    let view_logs = CustomMenuItem::new("view_logs", "📝 View Logs");
    let restart_service = CustomMenuItem::new("restart_service", "🔄 Restart Service");
    let separator3 = SystemTrayMenuItem::Separator;

    let quit = CustomMenuItem::new("quit", "🚪 Quit");

    let tray_menu = SystemTrayMenu::new()
        .add_item(start_recording)
        .add_item(stop_recording)
        .add_native_item(separator1)
        .add_item(status)
        .add_item(last_transcription)
        .add_native_item(separator2)
        .add_item(settings)
        .add_item(view_logs)
        .add_item(restart_service)
        .add_native_item(separator3)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

/// Handle system tray events
pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            debug!("System tray left click");
            // Show main window on left click
            if let Some(window) = app.get_window("main") {
                if let Err(e) = window.show() {
                    error!("Failed to show window: {}", e);
                }
                if let Err(e) = window.set_focus() {
                    error!("Failed to focus window: {}", e);
                }
            }
        }
        SystemTrayEvent::RightClick {
            position: _,
            size: _,
            ..
        } => {
            debug!("System tray right click");
            // Menu is shown automatically
        }
        SystemTrayEvent::DoubleClick {
            position: _,
            size: _,
            ..
        } => {
            debug!("System tray double click");
            // Show dashboard on double click
            if let Some(window) = app.get_window("main") {
                if let Err(e) = window.show() {
                    error!("Failed to show window: {}", e);
                }
                if let Err(e) = window.set_focus() {
                    error!("Failed to focus window: {}", e);
                }
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            debug!("Menu item clicked: {}", id);
            handle_menu_click(app, &id);
        }
        _ => {}
    }
}

/// Handle menu item clicks
fn handle_menu_click(app: &AppHandle, menu_id: &str) {
    match menu_id {
        "start_recording" => {
            debug!("Start recording clicked");
            // Emit event to frontend
            if let Err(e) = app.emit_all("recording-start", ()) {
                error!("Failed to emit recording-start event: {}", e);
            }
        }
        "stop_recording" => {
            debug!("Stop recording clicked");
            // Emit event to frontend
            if let Err(e) = app.emit_all("recording-stop", ()) {
                error!("Failed to emit recording-stop event: {}", e);
            }
        }
        "settings" => {
            debug!("Settings clicked");
            if let Some(window) = app.get_window("main") {
                if let Err(e) = window.show() {
                    error!("Failed to show window: {}", e);
                }
                if let Err(e) = window.set_focus() {
                    error!("Failed to focus window: {}", e);
                }
                // Navigate to settings page
                if let Err(e) = app.emit_all("navigate-to", "settings") {
                    error!("Failed to emit navigate event: {}", e);
                }
            }
        }
        "view_logs" => {
            debug!("View logs clicked");
            if let Some(window) = app.get_window("main") {
                if let Err(e) = window.show() {
                    error!("Failed to show window: {}", e);
                }
                if let Err(e) = window.set_focus() {
                    error!("Failed to focus window: {}", e);
                }
                // Navigate to logs page
                if let Err(e) = app.emit_all("navigate-to", "logs") {
                    error!("Failed to emit navigate event: {}", e);
                }
            }
        }
        "restart_service" => {
            debug!("Restart service clicked");
            if let Err(e) = app.emit_all("service-restart", ()) {
                error!("Failed to emit service-restart event: {}", e);
            }
        }
        "quit" => {
            debug!("Quit clicked");
            app.exit(0);
        }
        _ => {
            debug!("Unknown menu item: {}", menu_id);
        }
    }
}

/// Update tray menu based on state
pub fn update_tray_menu(app: &AppHandle, state: &SharedState) {
    let app_state = match state.lock() {
        Ok(s) => s.clone(),
        Err(e) => {
            error!("Failed to lock state: {}", e);
            return;
        }
    };

    let tray_handle = match app.tray_handle() {
        Some(h) => h,
        None => {
            error!("Failed to get tray handle");
            return;
        }
    };

    // Update recording buttons
    if app_state.recording {
        if let Err(e) = tray_handle.get_item("start_recording").set_enabled(false) {
            error!("Failed to disable start_recording: {}", e);
        }
        if let Err(e) = tray_handle.get_item("stop_recording").set_enabled(true) {
            error!("Failed to enable stop_recording: {}", e);
        }
    } else {
        if let Err(e) = tray_handle.get_item("start_recording").set_enabled(true) {
            error!("Failed to enable start_recording: {}", e);
        }
        if let Err(e) = tray_handle.get_item("stop_recording").set_enabled(false) {
            error!("Failed to disable stop_recording: {}", e);
        }
    }

    // Update status text
    let status_text = format!("📊 Status: {:?}", app_state.server_status);
    if let Err(e) = tray_handle.get_item("status").set_title(&status_text) {
        error!("Failed to update status text: {}", e);
    }

    // Update last transcription (truncate to 50 chars)
    let last_text = if app_state.last_transcription.is_empty() {
        "🔊 Last: ---".to_string()
    } else {
        let truncated = if app_state.last_transcription.len() > 50 {
            format!("{}...", &app_state.last_transcription[..47])
        } else {
            app_state.last_transcription.clone()
        };
        format!("🔊 Last: {}", truncated)
    };
    if let Err(e) = tray_handle.get_item("last_transcription").set_title(&last_text) {
        error!("Failed to update last transcription: {}", e);
    }
}

/// Update tray icon based on state
pub fn update_tray_icon(app: &AppHandle, tray_state: TrayState) {
    let icon_path = match tray_state {
        TrayState::Idle => "icons/icon-idle.png",
        TrayState::Listening => "icons/icon-listening.png",
        TrayState::Processing => "icons/icon-processing.png",
        TrayState::Error => "icons/icon-error.png",
        TrayState::Disabled => "icons/icon-disabled.png",
    };

    debug!("Updating tray icon to: {:?} ({})", tray_state, icon_path);

    // Note: In production, you would load the actual icon file
    // For now, we just log the change
    // app.tray_handle().set_icon(tauri::Icon::File(icon_path.into())).ok();
}

/// Set up tray tooltip
pub fn update_tray_tooltip(app: &AppHandle, text: &str) {
    if let Some(tray_handle) = app.tray_handle() {
        if let Err(e) = tray_handle.set_tooltip(text) {
            error!("Failed to update tray tooltip: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_menu_creation() {
        let tray = build_tray_menu();
        // Basic test - just ensure it doesn't panic
        assert!(true);
    }
}
