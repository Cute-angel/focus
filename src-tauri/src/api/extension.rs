use crate::api::command_tree::{CommandDispatcher, CommandNode};

#[derive(serde::Serialize, Debug, Clone)]
pub struct action {
    pub(crate) icon: String,
    pub(crate) tooltip: String,
    pub(crate) value: String,
    pub(crate) id:String,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct ExtensionResult {
    pub(crate) icon: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) actions: Vec<action>,
}

#[derive(serde::Serialize, Debug,Clone)]
pub struct Results {
    pub(crate) total_count: usize,
    pub(crate) items: Vec<ExtensionResult>,
}

pub struct extension_info {
    id: String,
    commands: Vec<CommandNode>,
    priority: usize,
}

impl extension_info {
    pub fn default(id: &str) -> Self {
        Self {
            id: id.to_string(),
            commands: vec![],
            priority: 100,
        }
    }

    pub fn add_command(mut self, command: CommandNode) -> Self {
        self.commands.push(command);
        self
    }

    pub fn set_priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

pub trait Extension {
    fn OnMount(&self, command_dispatcher: &mut CommandDispatcher);

    fn OnUnmount(&self, command_dispatcher: &mut CommandDispatcher);
}
