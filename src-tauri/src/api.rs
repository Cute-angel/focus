use global_hotkey::hotkey::HotKey;
use tauri::{App, Manager};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_global_shortcut::ShortcutState;

pub mod command_tree;
pub mod extension;
pub mod types;

#[cfg(desktop)]
pub fn register_globals_shortcut(app: &mut App,global_keys:HotKey) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, _shortcut, event| {
                if let ShortcutState::Pressed = event.state {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            })
            .build(),
    )?;
    app.global_shortcut().register(global_keys)?;
    Ok(())
}
