// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use crate::api::command_tree::CommandDispatcher;
use crate::api::extension::Extension;
use crate::extension::cal_plugin::Calculator;
use crate::extension::demo_plugin::DemoExtension;
use tauri::async_runtime::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use crate::commands::{query, run_action};
use crate::extension::app_plugin::AppPlugin;

mod api;
mod commands;
mod extension;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
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

            // register global short
            #[cfg(desktop)]
            {
                api::register_globals_shortcut(app)?;
            }

            // command_dispatcher
            let mut command_dispatcher = CommandDispatcher::new("/");
            let demo = DemoExtension::default();
            let cal = Calculator::default();
            let app_manager = AppPlugin::default();


            // let plugin_list: Vec<Box<dyn Extension>> = vec![
            //     Box::new(demo) as Box<dyn Extension>,  // 显式地转换为 Box<dyn Extension>
            //     Box::new(cal) as Box<dyn Extension>,
            // ];
            //
            // for i in &plugin_list{
            //     i.OnMount(&mut command_dispatcher);
            // }
            demo.OnMount(&mut command_dispatcher);
            cal.OnMount(&mut command_dispatcher);
            app_manager.OnMount(&mut command_dispatcher);

            app.manage(Mutex::new(command_dispatcher));

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            query,
            run_action
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
