use std::any::Any;
use std::collections::HashMap;

pub struct CommandNode {
    pub name: String,
    pub child: HashMap<String, CommandNode>,
    pub execute: Option<Box<dyn Fn(CommandContext) -> Box<dyn Any> + Send>>,
    pub truncation: bool,
}

impl CommandNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            child: HashMap::new(),
            execute: None,
            truncation: false,
        }
    }

    pub fn then(mut self, child: CommandNode) -> Self {
        self.child.insert(child.name.clone(), child);
        self
    }

    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: (Fn(CommandContext) -> Box<dyn Any>) + 'static + Send,
    {
        self.execute = Some(Box::new(f));
        self
    }

    pub fn set_truncate(mut self) -> Self {
        self.truncation = true;
        self
    }
}

pub struct CommandContext {
    ctx: Vec<String>,
    args: HashMap<String, Box<dyn std::any::Any>>,
}

impl CommandContext {
    pub fn default() -> Self {
        Self {
            ctx: Vec::new(),
            args: HashMap::new(),
        }
    }

    pub fn add_parm(&mut self, name: &str, arg: Box<dyn std::any::Any>) {
        self.args.insert(name.into(), arg);
    }

    pub fn get_parm(&self, name: &str) -> Option<&Box<dyn std::any::Any>> {
        self.args.get(name)
    }
}

pub struct CommandDispatcher {
    root: CommandNode,
}

impl CommandDispatcher {
    pub fn new(prefix: &str) -> Self {
        Self {
            root: CommandNode::new(prefix.to_string()),
        }
    }

    pub fn register(&mut self, child: CommandNode) {
        self.root.child.insert(child.name.clone(), child);
    }

    pub fn run(&mut self, input: String) -> Option<(&Box<dyn Fn(CommandContext) -> Box<(dyn std::any::Any + 'static)> + Send>, CommandContext)>  {
        let command_content;
        // cut the prefix
        if let Some(input) = input.strip_prefix(&self.root.name) {
            command_content = input;
        } else {
            return None;
        }

        let mut ctx = CommandContext::default();
        let part = command_content.trim().split(' ').collect::<Vec<&str>>();

        let mut current_node = &self.root;

        let mut iter = part.iter();

        for part in iter.by_ref() {
            if let Some(child) = current_node.child.get(*part) {
                current_node = child;
                if (current_node.truncation) {
                    let mut rest_part = part.trim().to_string();
                    rest_part.push(' ');
                    rest_part += &*iter
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");
                    ctx.ctx.push(rest_part.trim().to_string());
                    break;
                }
                ctx.ctx.push(part.trim().to_string());
            }
        }

        if let Some(func) = &current_node.execute {
            Some(( func ,ctx))
        }else{
            None
        }

    }
}

#[cfg(test)]
mod tests {
    use crate::api::command_tree::{CommandContext, CommandDispatcher, CommandNode};
    use std::any::Any;

    #[test]
    fn test_command_node() {
        let cmd =
            CommandNode::new("test".to_string()).then(CommandNode::new("args".to_string()).then(
                CommandNode::new("arg1".to_string()).execute(|f| {
                    assert_eq!(
                        f.ctx,
                        vec!["test".to_string(), "args".to_string(), "arg1".to_string()]
                    );
                    Box::new(()) as Box<dyn Any>
                }),
            ));
        let mut command_dispatcher = CommandDispatcher::new("/");
        command_dispatcher.register(cmd);

        command_dispatcher.run("/test args arg1".to_string());
    }

    #[test]
    fn test_truncation() {
        let func = |f: CommandContext| {
            assert_eq!(f.ctx[0], "test".to_string());
            assert_eq!(f.ctx[1], "first arg1 arg2".to_string());

            return Box::new(1) as Box<dyn Any>;
        };

        let cmd = CommandNode::new("test".to_string()).then(
            CommandNode::new("first".to_string())
                .set_truncate()
                .execute(func),
        );
        let mut command_dispatcher = CommandDispatcher::new("/");
        command_dispatcher.register(cmd);
        command_dispatcher.run("/test first arg1 arg2".to_string());
    }
}
