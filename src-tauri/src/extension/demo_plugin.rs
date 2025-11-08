use std::any::Any;
use std::process::Command;
use crate::api::command_tree::{CommandDispatcher, CommandNode,CommandContext};
use crate::api::extension::{extension_info, Extension, ExtensionResult, Results};

pub struct DemoExtension{
    pub(crate) info:extension_info
}

impl Default for DemoExtension {
    fn default() -> Self {
        Self{
            info: extension_info::default("demo-plugin")
        }
    }
}

impl Extension for DemoExtension {
    fn OnMount<>(&self, command_dispatcher: &mut CommandDispatcher) {
        let func = |ctx:CommandContext|{
            let str1 = String::from("this is a demo extension");


            let res =  ExtensionResult{
                icon:"a".to_string(),
                title:str1.clone(),
                description:str1.clone(),
                actions:Vec::new(),
            };

            return Box::new(res) as Box<dyn Any>;
        };



        let command = CommandNode::new("demo".to_string()).execute(func);


        command_dispatcher.register(command);
    }

    fn OnUnload<F: Fn()>(func: F) {
        todo!()
    }
}