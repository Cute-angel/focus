use std::path::PathBuf;
use std::vec;
use tauri::{AppHandle, Runtime, State};

use crate::api::action_runner::ActionRunner;
use crate::api::command_tree::CommandDispatcher;
use crate::api::extension::{ExtensionResult, Results};
use tauri::async_runtime::Mutex;
use crate::utils::{ IconExtractor};

// 创建在我们程序中可能发生的所有错误
#[derive(Debug, thiserror::Error)]
pub enum Error {

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
) -> Result<Results, String> {



    let mut dispatcher = dispatcher.lock().await;
    if let Some((func, ctx)) = dispatcher.run(input_text) {
        let a = func(ctx,app);
        if let Some(extension_result) = a.downcast_ref::<ExtensionResult>() {
            Ok(Results {
                total_count: 1,
                items: vec![extension_result.clone()],
            })
        }else if let Some(res) = a.downcast_ref::<Results>() {
            Ok(res.clone())
        } else {
            Ok(Results {
                total_count: 0,
                items: Vec::new(),
            })
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
    println!("{id}");
    let action_runner = ActionRunner ::get_instance();
    if let Some(action)=  action_runner.lock().unwrap().get(id.as_ref()){
        action(val,app);
    }


}

#[tauri::command]
pub async fn get_icon_res(){

}

#[tauri::command]
fn setup_focus_listener(window: tauri::Window) {}
