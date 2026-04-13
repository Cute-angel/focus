use std::str::FromStr;
use tauri::{AppHandle, Manager, Runtime};
use tauri::async_runtime::Mutex;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use global_hotkey::hotkey::HotKey;

use crate::api::command_tree::PluginError;
use crate::api::extension::Results;
use crate::core::Core;

// 创建在我们程序中可能发生的所有错误
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Plugin(#[from] PluginError),
    #[error("CoreError: {0}")]
    CoreError(String)
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupStatus {
    onboarding_completed: bool,
    current_hotkey: String,
}

#[derive(serde::Serialize)]
pub struct HotkeyResponse {
    hotkey: String,
}

#[tauri::command]
pub async fn query<R: Runtime>(
    app: AppHandle,
    _window: tauri::Window<R>,
    input_text: String,
) -> Result<Results, Error> {
    dbg!(&input_text);
    if let Some(core_state) = app.try_state::<Mutex<Core>>() {
        let core = core_state.lock().await;
        let r = core
            .handle_query(input_text.as_str(), app.clone())
            .await;


        r
    } else {
        Err(Error::CoreError(String::from("no core found")))
    }


}

#[tauri::command]
pub async fn run_action(
    id: String,
    val: String,
    app: AppHandle,
) -> Result<(), Error> {
    if let Some(core_state) = app.try_state::<Mutex<Core>>() {
        let core = core_state.lock().await;
        core.handle_action(id, val, app.clone()).await;
    }


    Ok(())
}

#[tauri::command]
pub async fn get_icon_res() {}

#[tauri::command]
fn setup_focus_listener(_window: tauri::Window) {}

#[tauri::command]
pub async fn get_startup_status() -> Result<StartupStatus, Error> {
    let app = crate::APP_HANDLE
        .get()
        .ok_or_else(|| Error::CoreError(String::from("app handle not initialized")))?;
    let core_state = app
        .try_state::<Mutex<Core>>()
        .ok_or_else(|| Error::CoreError(String::from("no core found")))?;
    let core = core_state.lock().await;

    Ok(StartupStatus {
        onboarding_completed: core.is_startup(),
        current_hotkey: core.get_global_hotkey_store().to_string(),
    })
}

#[tauri::command]
pub async fn set_global_hotkey(app: AppHandle, accelerator: String) -> Result<HotkeyResponse, Error> {
    let hotkey = HotKey::from_str(accelerator.as_str())
        .map_err(|err| Error::CoreError(format!("Invalid shortcut: {}", err)))?;

    let global_shortcut = app.global_shortcut();
    global_shortcut
        .unregister_all()
        .map_err(|err| Error::CoreError(format!("Failed to clear existing shortcut: {}", err)))?;
    global_shortcut
        .register(hotkey)
        .map_err(|err| Error::CoreError(format!("Unable to register shortcut: {}", err)))?;

    if let Some(core_state) = app.try_state::<Mutex<Core>>() {
        let mut core = core_state.lock().await;
        core.set_global_hotkey(hotkey);
        core.flag_started();
        core.shutdown();
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    Ok(HotkeyResponse {
        hotkey: hotkey.to_string(),
    })
}
