use std::error::Error;
use tauri::{App, Manager};
use tauri_plugin_global_shortcut::ShortcutState;

pub mod command_tree;
pub mod extension;
pub mod types;

#[cfg(desktop)]
pub fn register_globals_shortcut(app: &mut App) -> Result<(), Box<dyn Error>> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
    let ctrl_n_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyN);
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                println!("{:?}", shortcut);
                if shortcut == &ctrl_n_shortcut {
                    match event.state {
                        ShortcutState::Pressed => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        ShortcutState::Released => {}
                    }
                }
            })
            .build(),
    )?;
    app.global_shortcut().register(ctrl_n_shortcut)?;
    Ok(())
}
