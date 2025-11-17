use std::any::Any;
use tauri::AppHandle;
use crate::api::action_runner::{Action, ActionRunner, ACTION_RUNNER};
use crate::api::command_tree::{CommandContext, CommandDispatcher, CommandNode};
use crate::api::extension::{action, Extension, ExtensionResult, Results};

#[derive(Default)]
pub struct AppPlugin {}

impl AppPlugin {

    fn get_plugin_action(&self) -> Action {
        let func = |val:String,app:AppHandle|{
            match val.as_str() {
                "restart"=> app.restart(),
                "stop" => app.exit(0),
                _ => {}
            };
        };
        Box::new(func)
    }


    fn get_commands(&self) -> CommandNode{
        let icon = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5"
                stroke="currentColor" class="size-8 my-auto">
                <path stroke-linecap="round" stroke-linejoin="round"
                    d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
            </svg>"#.to_string();
        let restart_res =  ExtensionResult {
            icon: icon.clone(),
            title: "Restart".to_string(),
            description: "Restart".to_string(),
            actions: vec![action {
                icon: "hide".to_string(),
                tooltip: "Enter".to_string(),
                value: "restart".to_string(),
                id:"app_manager".to_string()

            }],
        };


        let stop_res = ExtensionResult {
            icon,
            title: "Stop".to_string(),
            description: "Stop app".to_string(),
            actions: vec![action {
                icon: "hide".to_string(),
                tooltip: "Enter".to_string(),
                value: "stop".to_string(),
                id:"app_manager".to_string()

            }],
        };

        let value = restart_res.clone();
        let value2 = stop_res.clone();
        let show_restart_app = move |_ctx:CommandContext,_app:AppHandle| {
            Box::new(restart_res.clone()) as Box<dyn Any>
        };
        let show_stop_app = move |ctx:CommandContext,app:AppHandle| {
            Box::new(stop_res.clone()) as Box<dyn Any>
        };

        let show_app = move |ctx,_|{
            let res = Results {
                total_count:2,
                items:vec![value.clone(),value2.clone()],
            };
            Box::new(res) as Box<dyn Any>
        };

        let func_stop_app = |ctx:CommandContext,app:AppHandle| {

        };


        CommandNode::new("manager").then(
            CommandNode::new("restart").execute(show_restart_app)
        ).then(
            CommandNode::new("stop").execute(show_stop_app)
        ).execute(
            show_app
        )
    }
}

impl Extension for AppPlugin {


    fn OnMount(&self, command_dispatcher: &mut CommandDispatcher) {
        command_dispatcher.register(
            self.get_commands()
        );

        let action_runner = ActionRunner::get_instance();
        action_runner.lock().unwrap().add("app_manager",self.get_plugin_action());

    }



    fn OnUnmount(&self, command_dispatcher: &mut CommandDispatcher) {
        todo!()
    }
}