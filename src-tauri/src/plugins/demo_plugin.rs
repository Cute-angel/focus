use crate::api::command_tree::{CommandContext, CommandDispatcher, CommandNode, StringArgument};
use crate::api::extension::{Extension, ExtensionResult, MetaData, Results};
use crate::core::Core;

pub struct DemoPlugin {
    pub(crate) info: MetaData,
}

impl Default for DemoPlugin {
    fn default() -> Self {
        Self {
            info: MetaData::default_builder("demo-plugin"),
        }
    }
}

impl Extension for DemoPlugin {
    fn get_meta_data(&self) -> MetaData {
        self.info.clone()
    }

    fn on_plugin_load(&self, core: &mut Core) {
        let func = |ctx: CommandContext, _| {
            let str1 = String::from("this is a demo plugins");

            let res = ExtensionResult {
                icon: "a".to_string(),
                title: str1.clone(),
                description: ctx.get_parm("demo-args").unwrap().to_string(),
                actions: Vec::new(),
            };
            println!("{:?}", res);

            res.into()
        };

        let command = CommandNode::new("demo").then(
            CommandNode::new("demo-args").argument(StringArgument).then(
                CommandNode::new("nums")
                    .argument(StringArgument)
                    .execute(func),
            ),
        );
        core.get_command_dispatcher().register(command);
    }
}
