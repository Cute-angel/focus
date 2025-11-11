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
        let restart_res =  ExtensionResult {
            icon: "a".to_string(),
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
            icon: "a".to_string(),
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


        CommandNode::new("app").then(
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