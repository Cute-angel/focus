use std::any::Any;
use std::collections::HashMap;
use tauri::AppHandle;

pub trait Parameter {
    fn parse(&self, _: &str) -> Result<String, String>;
}

pub enum NodeType {
    Literal,
    Parameter(Option<Box<dyn Parameter + Send>>),
}

pub struct StringArgument;

impl Parameter for StringArgument {
    fn parse(&self, input: &str) -> Result<String, String> {
        Ok(input.to_string())
    }
}

pub struct CommandNode {
    pub name: String,
    pub child: HashMap<String, CommandNode>,
    pub execute: Option<Box<dyn Fn(CommandContext,AppHandle) -> Box<dyn Any> + Send>>,
    pub node_type: NodeType,
    pub truncation: bool,
}

impl Default for CommandNode {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            child: HashMap::new(),
            execute: None,
            node_type: NodeType::Literal,
            truncation: false,
        }
    }
}

impl CommandNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            child: HashMap::new(),
            execute: None,
            truncation: false,
            // when argument set to None the
            node_type: NodeType::Literal,
        }
    }

    pub fn then(mut self, child: CommandNode) -> Self {
        self.child.insert(child.name.clone(), child);
        self
    }

    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: (Fn(CommandContext,AppHandle) -> Box<dyn Any>) + 'static + Send,
    {
        self.execute = Some(Box::new(f));
        self
    }

    pub fn set_truncate(mut self) -> Self {
        self.truncation = true;
        self
    }

    pub fn argument<T: Parameter + 'static + std::marker::Send>(mut self, arg: T) -> Self {
        self.node_type = NodeType::Parameter(Some(Box::new(arg)));
        self
    }
}

pub struct CommandContext {
    ctx: Vec<String>,
    args: HashMap<String, Box<String>>,
}

impl CommandContext {
    pub fn default() -> Self {
        Self {
            ctx: Vec::new(),
            args: HashMap::new(),
        }
    }

    pub fn add_parm(&mut self, name: &str, arg: Box<String>) {
        self.args.insert(name.into(), arg);
    }

    pub fn get_parm(&self, name: &str) -> Option<&Box<String>> {
        self.args.get(name)
    }
}

pub struct CommandDispatcher {
    root: CommandNode,
}

impl CommandDispatcher {
    pub fn new(prefix: &str) -> Self {
        Self {
            root: CommandNode::new(prefix),
        }
    }

    pub fn register(&mut self, child: CommandNode) {
        self.root.child.insert(child.name.clone(), child);
    }

    pub fn run(
        &mut self,
        input: String,
    ) -> Option<(
        &Box<dyn Fn(CommandContext,AppHandle) -> Box<(dyn Any + 'static)> + Send>,
        CommandContext,
    )> {
        let command_content;
        // cut the prefix and match if the input start with prefix
        if let Some(input) = input.strip_prefix(&self.root.name) {
            command_content = input;
        } else {
            return None;
        }

        let mut ctx = CommandContext::default();
        let part = command_content
            .trim()
            .split_whitespace()
            .collect::<Vec<&str>>();

        let mut current_node = &self.root;

        let mut iter = part.iter();

        for part in iter.by_ref() {
            // always start with Literal
            if let Some(child) = current_node.child.get(*part) {
                // when part match node name and type is literal
                if let NodeType::Literal = child.node_type {
                    current_node = child;
                    // literal node doesn't need record val
                    continue;
                }
            }
            // try to match Parameter
            let mut matched = false;
            for i in current_node.child.values() {
                if let NodeType::Parameter(ref arg_type) = &i.node_type {
                    if current_node.truncation {
                        current_node = i;
                        matched = true;
                        break;
                    }

                    if let Some(arg) = arg_type {
                        match arg.parse(*part) {
                            Ok(value) => {
                                ctx.add_parm(i.name.as_ref(), Box::new(value));
                                matched = true;
                                current_node = i;
                                break;
                            }
                            Err(_) => {
                                println!("Error parsing parameter \"{}\"", part.trim());
                            }
                        }
                    }
                }
            }
            if !matched {
                println!("Unknown command or argument: {}", part);
                return None;
            }
            if (current_node.truncation) {
                let mut rest_part = part.trim().to_string();
                rest_part.push(' ');
                rest_part += &*iter
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                ctx.add_parm(current_node.name.as_ref(), Box::from(rest_part));
                break;
            }
            ctx.ctx.push(part.trim().to_string());
        }

        if let Some(func) = &current_node.execute {
            Some((func, ctx))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::command_tree::{
        CommandContext, CommandDispatcher, CommandNode, NodeType, StringArgument,
    };
    use std::any::Any;
    use tauri::AppHandle;


    fn dummy_app() -> AppHandle {
        use tauri::Manager;

        tauri::Builder::default()
            .build(tauri::generate_context!())
            .expect("failed to build dummy tauri app")
            .app_handle().clone()
    }

    #[test]
    fn test_command_node() {
        let app = dummy_app();

        let cmd = CommandNode::new("test").then(
            CommandNode::new("args").then(
                CommandNode::new("arg1")
                    .argument(StringArgument)
                    .execute(|f,_app| {
                        assert_eq!(f.get_parm("arg1"), Some(&Box::new(String::from("123"))));
                        assert_eq!(f.get_parm("args"), None);

                        Box::new(()) as Box<dyn Any>
                    }),
            ),
        );
        let mut command_dispatcher = CommandDispatcher::new("/");
        command_dispatcher.register(cmd);

        if let Some((_func, ctx)) = command_dispatcher.run("/test        args 123".to_string()) {
            assert_eq!(ctx.get_parm("arg1"), Some(&Box::new(String::from("123"))));
            assert_eq!(ctx.get_parm("args"), None);
        }
    }

    #[test]
    fn test_truncation() {
        let func = |f: CommandContext,_app| {
            assert_eq!(
                f.get_parm("arg1"),
                Some(&Box::new("first arg1 arg2".to_string()))
            );
            return Box::new(1) as Box<dyn Any>;
        };

        let cmd = CommandNode::new("test").then(
            CommandNode::new("first")
                .argument(StringArgument)
                .set_truncate()
                .execute(func),
        );
        let mut command_dispatcher = CommandDispatcher::new("/");
        command_dispatcher.register(cmd);

        if let Some((_func, ctx)) =
            command_dispatcher.run("/test            first arg1 arg2".to_string())
        {
            assert!(ctx.get_parm("arg1").is_none());
            assert!(ctx.get_parm("arg2").is_none());
            assert_eq!(
                ctx.get_parm("first"),
                Some(&Box::new("first arg1 arg2".to_string()))
            );
            println!("{}", ctx.get_parm("first").unwrap());
        }
    }
}
