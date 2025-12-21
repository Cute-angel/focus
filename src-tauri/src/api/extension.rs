use std::cmp::Ordering;
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

#[derive(Debug, Clone)]
pub struct MetaData {
    id: String,
    version: String,
    priority: usize,
}

impl MetaData {
    pub fn default_builder(id: &str) -> Self {
        Self {
            id: id.to_string(),
            priority: 100,
            version: "1.0.0".to_string(),
        }
    }

    pub fn set_priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }

    pub fn set_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    pub fn build(self) -> Self {
        self
    }

}

pub trait Extension {
    fn OnMount(&self, command_dispatcher: &mut CommandDispatcher);

    fn OnUnmount(&self, command_dispatcher: &mut CommandDispatcher);

    fn get_meta_data(&self) -> MetaData;
}

impl PartialOrd for dyn Extension {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.get_meta_data().priority.cmp(&other.get_meta_data().priority)
        )
    }
}

impl PartialEq for dyn Extension {
    fn eq(&self, other: &Self) -> bool {
        self.get_meta_data().id == other.get_meta_data().id
    }
}
