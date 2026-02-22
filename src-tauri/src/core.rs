pub mod action_runner;
mod config_helper;
mod plugin_manager;
mod plugin_worker;
mod shortcut;

pub use crate::core::shortcut::ScoredItem;
use crate::api::command_tree::CommandDispatcher;
use crate::api::extension::{Extension, Results};
use crate::api::types::PluginResult;
use crate::commands::Error;
use crate::core::action_runner::ActionRunner;
use crate::core::config_helper::ConfigHelper;
use crate::core::plugin_manager::PluginManager;
use crate::core::shortcut::ShortcutsDispatcher;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::mem::take;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use tauri::AppHandle;

pub struct Core {
    extension_lt: Vec<Box<dyn Extension>>,
    command_dispatcher: CommandDispatcher,
    shortcut_dispatcher: ShortcutsDispatcher<AppHandle, PluginResult>,
    action_runner: ActionRunner,
    config_helper: ConfigHelper,
}


static CORE: AtomicPtr<Core> = AtomicPtr::new(null_mut());
static REF_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static CORE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

impl Core {
    pub fn new()  {
        if !CORE.load(Ordering::SeqCst).is_null() {
            return;
        }
        let mut config_helper = ConfigHelper::default();
        let _ = &config_helper.load();
        let command_dispatcher =
            CommandDispatcher::new(config_helper.get_value("command_prefix", "/".to_string()));
        let shortcut_dispatcher = ShortcutsDispatcher::<AppHandle, PluginResult>::new();
        let mut core = Self {
            extension_lt: PluginManager::get_builtin_plugins(),
            action_runner: ActionRunner::new(),
            command_dispatcher,
            config_helper,
            shortcut_dispatcher,
        };
        core.init();
        let c;
        unsafe {
             c =  Box::leak(Box::new(core));

        }
        CORE.store(c as *mut Core,Ordering::Release);

    }

    pub fn  get_instance() -> &'static Core {
       if let Some(c) =  unsafe { CORE.load(Ordering::SeqCst).as_ref() }{
           REF_COUNT.fetch_add(1, Ordering::Relaxed);
           c
       }else {
           panic!("Core instance not initialized")
       }

    }

    pub fn sub_ref() {
        REF_COUNT.fetch_sub(1,Ordering::Release);
    }

    pub fn free(){
        if  dbg!(REF_COUNT.load(Ordering::Relaxed)) > 0{
            panic!("Core instance has refence")
        }

        unsafe {
            let _ = Box::from_raw(CORE.swap(null_mut(), Ordering::SeqCst));
        }
        REF_COUNT.store(0, Ordering::SeqCst);
    }

    pub fn init(&mut self) {
        // init core
        if !self.config_helper.get_value("init", false)
            || self
                .config_helper
                .get_value("version", CORE_VERSION.to_string())
                .as_str()
                != CORE_VERSION
        {
            self.config_helper.set_value("", CoreConfig::default()).expect("TODO: panic message");
        }
        // init plugins
        let extensions = std::mem::take(&mut self.extension_lt);
        for i in &extensions {
            i.on_plugin_load(self);
        }
        self.extension_lt = extensions;
    }

    pub async fn handle_query(&self, text: &str, app: AppHandle) -> Result<Results, Error> {
        let mut plugin_result;
        if let Some((func, ctx)) = self.command_dispatcher.run(text.to_string()) {
            plugin_result = func(ctx, app);
        } else if text.contains("/") {
            plugin_result = PluginResult::Results(Results {
                total_count: 0,
                items: Vec::new(),
            });
        }
        else
        {
             plugin_result = self.shortcut_dispatcher.run(text, app, Some(10)).await.into();
        }
        match plugin_result {
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
    }
    pub async fn handle_action(&self, id: String,val:String, app: AppHandle) {
        if let Some(action) = self.action_runner.get(id.as_ref()) {
            action(val, app)
        }
    }

    pub fn get_command_dispatcher(&mut self) -> &mut CommandDispatcher {
        &mut self.command_dispatcher
    }

    pub fn get_shortcut_dispatcher(&mut self) -> &mut ShortcutsDispatcher<AppHandle, PluginResult> {
        &mut self.shortcut_dispatcher
    }

    pub fn get_action_runner(&mut self) -> &mut ActionRunner {
        &mut self.action_runner
    }

    pub fn add_extension(mut self, ext: Box<dyn Extension>) -> Self {
        self.extension_lt.push(ext);
        self
    }

    pub fn get_config<T>(&self,plugin: impl Extension,namespace:&str,default:T)->T
    where T: DeserializeOwned + Serialize
    {
        let  namespace = plugin.get_meta_data().id + "." + namespace;
        self.config_helper.get_value(&*namespace, default)
    }

    pub fn get_config_value<T>(&mut self, plugin: impl Extension, namespace:&str, value:T) ->Result<() ,Box<dyn std::error::Error + '_>>

    where T: DeserializeOwned + Serialize
    {
        let namespace = plugin.get_meta_data().id + "." + namespace;
         Ok( self.config_helper.set_value(&*namespace, value)?)
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        take(&mut self.config_helper);
    }
}
#[derive(Serialize, Deserialize)]
struct CoreConfig {
    command_prefix: String,
    init: bool,
    version: String,
}

impl Default for CoreConfig {
    fn default() -> Self {
        CoreConfig {
            command_prefix: "/".to_string(),
            init: true,
            version: CORE_VERSION.to_string(),
        }
    }
}
