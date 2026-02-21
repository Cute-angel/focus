use crate::api::extension::Extension;
use crate::plugins::AppPlugin;
use crate::plugins::CalculatorPlugin;
use crate::plugins::DemoPlugin;
use crate::plugins::FilePlugin;
use crate::plugins::LauncherPlugin;

pub struct PluginManager {}

impl PluginManager {
    pub fn get_builtin_plugins() -> Vec<Box<dyn Extension>> {
        let mut plugins: Vec<Box<dyn Extension>> = Vec::new();
        plugins.push(Box::new(FilePlugin::default()));
        plugins.push(Box::new(AppPlugin::default()));
        plugins.push(Box::new(DemoPlugin::default()));
        plugins.push(Box::new(CalculatorPlugin::default()));
        plugins.push(Box::new(LauncherPlugin::default()));
        plugins
    }
}
