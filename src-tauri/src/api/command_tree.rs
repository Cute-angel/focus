use std::collections::HashMap;
use serde::forward_to_deserialize_any;
use tauri::AppHandle;
use thiserror::Error;
use crate::api::extension::{ExtensionResult, Results};

#[derive(Debug, Error)]
pub enum  PluginError{
    #[error("{0}:{1}")]
    Error(String,String),
}

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


pub enum  PluginResult {
    ExtensionResult(ExtensionResult),
    Results(Results),
    PluginError(PluginError),
    Null
}

impl From<ExtensionResult> for PluginResult {
    fn from(result: ExtensionResult) -> Self {
        PluginResult::ExtensionResult(result)
    }
}

impl From<Results> for PluginResult {
    fn from(result: Results) -> Self {
        PluginResult::Results(result)
    }
}


impl From<PluginError> for PluginResult {
    fn from(error: PluginError) -> Self {
        PluginResult::PluginError(error)
    }
}

pub type Callback =
Box<dyn Fn(CommandContext, AppHandle) -> PluginResult + Send >;



pub struct CommandNode {
    pub name: String,
    pub child: HashMap<String, CommandNode>,
    pub execute: Option<Callback>,
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
    where F: Fn(CommandContext, AppHandle) -> PluginResult + Send + 'static
    {
        self.execute = Some(Box::new(f));
        self
    }

    pub fn set_truncate(mut self) -> Self {
        self.truncation = true;
        self
    }

    pub fn argument<T: Parameter + 'static + Send>(mut self, arg: T) -> Self {
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
        &Callback,
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
            // get from hashmap ,try to map
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
                                eprintln!("Error parsing parameter \"{}\"", part.trim());
                            }
                        }
                    }
                }
            }
            if !matched {
                dbg!("Unknown command or argument: {}", part);
                return None;
            }
            // when truncation catch rest
            if (current_node.truncation) {
                // current
                let mut rest_part = part.trim().to_string();
                // add the blank deleted before
                rest_part.push(' ');
                //add the rest
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

