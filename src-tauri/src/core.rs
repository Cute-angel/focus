pub mod action_runner;
mod config_helper;
mod plugin_manager;
mod plugin_worker;
mod shortcut;

use crate::api::command_tree::CommandDispatcher;
use crate::api::extension::{Extension, Results};
use crate::api::types::PluginResult;
use crate::commands::Error;
use crate::core::action_runner::ActionRunner;
use crate::core::config_helper::ConfigHelper;
use crate::core::plugin_manager::PluginManager;
use crate::core::shortcut::ShortcutsDispatcher;
use std::ptr::null_mut;
use tauri::AppHandle;

pub struct Core {
    extension_lt: Vec<Box<dyn Extension>>,
    command_dispatcher: CommandDispatcher,
    shortcut_dispatcher: ShortcutsDispatcher<AppHandle, ()>,
    action_runner: ActionRunner,
    config_helper: ConfigHelper,
}

pub static mut CORE: *mut Core = null_mut::<Core>();

impl Core {
    pub fn new() {
        let mut config_helper = ConfigHelper::default();
        let _ = &config_helper.load();
        let command_dispatcher =
            CommandDispatcher::new(config_helper.get_value("command_prefix", "/".to_string()));
        let shortcut_dispatcher = ShortcutsDispatcher::<AppHandle, ()>::new();
        let core = Self {
            extension_lt: PluginManager::get_builtin_plugins(),
            action_runner: ActionRunner::new(),
            command_dispatcher,
            config_helper,
            shortcut_dispatcher,
        };

        unsafe {
            let c = Box::leak(Box::new(core));
            CORE = c as *mut Core;
        }
    }
    pub fn get_instance() -> &'static mut Core {
        if let Some(core) = unsafe { CORE.as_mut() } {
            core
        } else {
            panic!("Core not initialized")
        }
    }

    pub fn init(&mut self) {
        // init core

        // init plugins
        let extensions = std::mem::take(&mut self.extension_lt);
        for i in &extensions {
            i.on_plugin_load(self);
        }
        self.extension_lt = extensions;
    }

    pub async fn handle_query(&mut self, text: &str, app: AppHandle) -> Result<Results, Error> {
        if let Some((func, ctx)) = self.command_dispatcher.run(text.to_string()) {
            match func(ctx, app) {
                PluginResult::Null => Ok(Results {
                    total_count: 0,
                    items: Vec::new(),
                }),
                PluginResult::ExtensionResult(res) => Ok(Results {
                    total_count: 1,
                    items: vec![res.clone()],
                }),
                PluginResult::Results(res) => Ok(res.clone()),
                PluginResult::PluginError(err) => Err(Error::Plugin(err)),
            }
        } else {
            self.shortcut_dispatcher.run(text, app, None).await;
            Ok(Results {
                total_count: 0,
                items: Vec::new(),
            })
        }
    }

    pub fn get_command_dispatcher(&mut self) -> &mut CommandDispatcher {
        &mut self.command_dispatcher
    }

    pub fn get_shortcut_dispatcher(&mut self) -> &mut ShortcutsDispatcher<AppHandle, ()> {
        &mut self.shortcut_dispatcher
    }

    pub fn get_action_runner(&mut self) -> &mut ActionRunner {
        &mut self.action_runner
    }

    pub fn add_extension(mut self, ext: Box<dyn Extension>) -> Self {
        self.extension_lt.push(ext);
        self
    }
}
