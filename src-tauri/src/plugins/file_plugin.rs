use std::any::Any;
use std::sync::{Arc, LazyLock, Mutex};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::api::action_runner::{Action, ActionRunner};
use crate::api::command_tree::{
    Callback, CommandContext, CommandDispatcher, CommandNode, StringArgument,
};
use crate::api::extension::{action, Extension, ExtensionResult, MetaData, Results};
use crate::utils::EverythingHelper;
use crate::utils::to_base64;
use crate::utils::IconExtractor;
use futures::executor::block_on;
use crate::api::types::PluginResult;

pub struct FilePlugin {}

static EVERY_THING_HELPER: LazyLock<Arc<Mutex<EverythingHelper>>> =
    LazyLock::new(|| Arc::new(Mutex::new(EverythingHelper::default())));

impl Default for FilePlugin {
    fn default() -> Self {
        FilePlugin {}
    }
}
impl FilePlugin {
    fn get_action_icon(&self) {}

    fn get_show_result_func(&self) -> Callback {
        let async_func = async |ctx: CommandContext,
                                app: AppHandle|
               -> PluginResult {
            let helper = &*EVERY_THING_HELPER;
            if let Some(str) = ctx.get_parm("file_name") {
                let info = helper.lock().unwrap().query(str).await;

                let result_list = info.iter().map(
                   |item|{
                       let mut icon = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h15A2.25 2.25 0 0 1 21.75 12v.75m-8.69-6.44-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z" />
                                    </svg>"#.to_string();
                       if item.is_file() {
                           if let Some(data) = IconExtractor::default().get_icon(&item.get_path()){
                               icon = format!("<img src=\"data:image/png;base64,{}\" alt=\"Image\" />", to_base64(data))
                           }
                       }


                       ExtensionResult {
                           icon,
                           title:item.get_name(),
                           description:item.get_path().to_str().unwrap().to_string(),
                           actions:vec![
                                action{
                                    icon:"hide".to_string(),
                                    id:"file_plugin_runner".to_string(),
                                    tooltip:"1".to_string(),
                                    value:item.get_path().to_str().unwrap().to_string()
                                }
                           ]
                       }
                   }
               ).collect::<Vec<ExtensionResult>>();
                let result = Results {
                    total_count: result_list.len(),
                    items: result_list,
                };
                //dbg!(&result);
                result.into()
            } else {
                PluginResult::null
            }
        };
        Box::new(move |ctx, app| -> PluginResult {
            let fut = async_func(ctx, app);
            block_on(fut)
        })
    }
    fn get_nodes(&self) -> CommandNode {
        let node1 = CommandNode::new("file").then(
            CommandNode::new("file_name")
                .argument(StringArgument)
                .set_truncate()
                .execute(self.get_show_result_func()),
        );
        node1
    }

    fn get_action(&self) -> Action {
        let f = |val: String, app: AppHandle| {
            let r = match app.opener().open_path(val, None::<&str>) {
                Ok(_) => {}
                Err(_) => {}
            };
        };
        Box::new(f)
    }
}

impl Extension for FilePlugin {
    fn OnMount(&self, command_dispatcher: &mut CommandDispatcher) {
        command_dispatcher.register(self.get_nodes());

        let action_runner = ActionRunner::get_instance();
        action_runner
            .lock()
            .unwrap()
            .add("file_plugin_runner", self.get_action());
    }

    fn OnUnmount(&self, command_dispatcher: &mut CommandDispatcher) {
        todo!()
    }

    fn get_meta_data(&self) -> MetaData {
        MetaData::default_builder("FileSearcher").build()
    }
}
