use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime, State};

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

#[tauri::command]
pub async fn query<R: Runtime>(
    app: AppHandle,
    window: tauri::Window<R>,
    input_text: String,
) -> Result<Results, Error> {
    dbg!(&input_text);
    if let Some(core) = app.try_state::<Core>() {
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
    if let Some(core) = app.try_state::<Core>() {
        core.handle_action(id, val, app.clone()).await;
    }


    Ok(())
}

#[tauri::command]
pub async fn get_icon_res() {}

#[tauri::command]
fn setup_focus_listener(window: tauri::Window) {}
