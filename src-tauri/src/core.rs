mod plugin_manager;
mod plugin_worker;
mod config_helper;
mod shortcut;
pub mod action_runner;

use std::collections::HashMap;
use crate::api::command_tree::{Callback, CommandDispatcher};
use crate::api::extension::Extension;
use shortcut::ShortcutsDispatcher;
use crate::core::action_runner::ActionRunner;

pub struct Core{
    extension_lt:Vec<Box<dyn Extension>>,
    command_dispatcher: CommandDispatcher,
    shortcut_dispatcher :ShortcutsDispatcher,
    action_runner:ActionRunner,
}


impl Core {


    pub fn get_command_dispatcher(&mut self) -> &mut CommandDispatcher {
        &mut self.command_dispatcher
    }

    pub fn get_shortcut_dispatcher(&mut self) -> &mut ShortcutsDispatcher {
        &mut self.shortcut_dispatcher
    }


    pub fn add_extension(mut self, ext: Box<dyn Extension>) -> Self {
        self.extension_lt.push(ext);
        self
    }
}