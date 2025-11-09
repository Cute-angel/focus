use crate::api::command_tree::{CommandContext, CommandDispatcher, CommandNode, StringArgument};
use crate::api::extension::{extension_info, Extension, ExtensionResult, Results};
use std::any::Any;
use std::process::Command;

pub struct DemoExtension {
    pub(crate) info: extension_info,
}

impl Default for DemoExtension {
    fn default() -> Self {
        Self {
            info: extension_info::default("demo-plugin"),
        }
    }
}

impl Extension for DemoExtension {
    fn OnMount(&self, command_dispatcher: &mut CommandDispatcher) {
        let func = |ctx: CommandContext| {
            let str1 = String::from("this is a demo extension");

            let res = ExtensionResult {
                icon: "a".to_string(),
                title: str1.clone(),
                description: ctx.get_parm("demo-args").unwrap().to_string(),
                actions: Vec::new(),
            };
            println!("{:?}", res);

            return Box::new(res) as Box<dyn Any>;
        };

        let command = CommandNode::new("demo").then(
            CommandNode::new("demo-args").argument(StringArgument).then(
                CommandNode::new("nums")
                    .argument(StringArgument)
                    .execute(func),
            ),
        );

        command_dispatcher.register(command);
    }

    fn OnUnmount(&self, command_dispatcher: &mut CommandDispatcher) {
        todo!()
    }
}
