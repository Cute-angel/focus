use std::vec;
use tauri::{AppHandle, Runtime, State};

use crate::core::action_runner::ActionRunner;
use crate::api::command_tree::{CommandDispatcher, PluginError};
use crate::api::extension::Results;
use crate::api::types::PluginResult;
use tauri::async_runtime::Mutex;

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
    app: tauri::AppHandle,
    window: tauri::Window<R>,
    input_text: String,
    dispatcher: State<'_, Mutex<CommandDispatcher>>,
) -> Result<Results, Error> {

    let mut dispatcher = dispatcher.lock().await;
    if let Some((func, ctx)) = dispatcher.run(input_text) {
        match func(ctx,app) {
            PluginResult::Null => {
                Ok(Results {
                    total_count: 0,
                    items: Vec::new(),
                })
            }
            PluginResult::ExtensionResult(res) => {
                Ok(Results {
                    total_count: 1,
                    items: vec![res.clone()],
                })
            }
            PluginResult::Results(res) => {
                Ok(res.clone())
            }
            PluginResult::PluginError(err) => {
                Err(Error::Plugin(err))
            }
        }


    } else {
        Ok(Results {
            total_count: 0,
            items: Vec::new(),
        })
    }
}

#[tauri::command]
pub fn run_action(id: String, val:String, app:AppHandle ) {
    dbg!("{id}");
    let action_runner = ActionRunner::get_instance();
    if let Some(action)=  action_runner.lock().unwrap().get(id.as_ref()){
        action(val,app);
    }
}

#[tauri::command]
pub async fn get_icon_res(){

}

#[tauri::command]
fn setup_focus_listener(window: tauri::Window) {}
