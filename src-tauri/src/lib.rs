// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::sync::{Arc, OnceLock};
use crate::api::command_tree::CommandDispatcher;
use crate::api::extension::Extension;
use crate::plugins::CalculatorPlugin;
use crate::plugins::DemoPlugin;
use tauri::async_runtime::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};
use crate::commands::{query, run_action};
use crate::plugins::AppPlugin;
use crate::plugins::FilePlugin;
use crate::plugins::LauncherPlugin;
mod api;
mod commands;
mod plugins;
pub mod utils;
mod core;

static APP_HANDLE: OnceLock<Arc<AppHandle>> = OnceLock::new();
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let hide_i = MenuItem::with_id(app, "hide", "隐藏focus", true, None::<&str>)?;
            let open_query_page =
                MenuItem::with_id(app, "query_open", "显示focus", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app,"quit","退出App",true,None::<&str>)?;

            let menu = Menu::with_items(app, &[&quit_i, &hide_i,&open_query_page])?;

            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                        println!("Quit");
                    }
                    "quit" =>{
                        app.exit(0);
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
            let demo = DemoPlugin::default();
            let cal = CalculatorPlugin::default();
            let app_manager = AppPlugin::default();
            let file = FilePlugin::default();
            let launcher = LauncherPlugin::default();
            let _ = launcher.init();

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
            file.OnMount(&mut command_dispatcher);
            launcher.OnMount(&mut command_dispatcher);

            app.manage(Mutex::new(command_dispatcher));

            APP_HANDLE.set(Arc::new(app.handle().clone())).ok();
            Ok(())
        })

        .invoke_handler(tauri::generate_handler![
            query,
            run_action
        ]).build(tauri::generate_context!()).expect("error in build app")
        .run(|app_handle, event| {          // 运行阶段，使用 AppHandle
            match event {
                tauri::RunEvent::ExitRequested { .. } => {
                    println!("用户请求退出");
                }
                tauri::RunEvent::Exit => {

                    println!("应用退出");
                }
                _ => {}
            }
        })

}
