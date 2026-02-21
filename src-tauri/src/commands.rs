use tauri::{AppHandle, Runtime};

use crate::api::command_tree::PluginError;
use crate::api::extension::Results;
use crate::core::Core;

// 创建在我们程序中可能发生的所有错误
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Plugin(#[from] PluginError),
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
    Core::get_instance()
        .handle_query(input_text.as_str(), app)
        .await
}

#[tauri::command]
pub fn run_action(id: String, val: String, app: AppHandle) {
    dbg!(&id);
    let action_runner = Core::get_instance().get_action_runner();
    if let Some(action) = action_runner.get(id.as_ref()) {
        action(val, app);
    }
}

#[tauri::command]
pub async fn get_icon_res() {}

#[tauri::command]
fn setup_focus_listener(window: tauri::Window) {}
