// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use tauri::async_runtime::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use crate::api::command_tree::CommandDispatcher;
use crate::api::extension::Extension;
use crate::extension::demo_plugin::DemoExtension;
mod api;
mod commands;
mod extension;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let open_query_page =
                MenuItem::with_id(app, "query_open", "显示focus", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i, &open_query_page])?;

            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                        println!("Quit");
                    }
                    "query_open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    &_ => todo!(),
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            // command_dispatcher
            let mut command_dispatcher = CommandDispatcher::new("/");
            let demo =  DemoExtension::default();
            demo.OnMount(&mut command_dispatcher);

            app.manage(Mutex::new(command_dispatcher));

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::query,
            commands::run_action
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
