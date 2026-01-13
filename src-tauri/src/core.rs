mod plugin_manager;

use std::collections::HashMap;
use crate::api::command_tree::{Callback, CommandDispatcher};
use crate::api::extension::Extension;
use crate::commands::Error;

pub struct ShortCutDispatcher{
    store:HashMap<char,Vec<Callback>>,
    any:Vec<Callback>,
}

impl Default for ShortCutDispatcher {
    fn default() -> Self {
        Self {
            store:HashMap::with_capacity(4),
            any:Vec::with_capacity(4),
        }
    }
}

impl ShortCutDispatcher {


    pub fn add_shortcut(&mut self, shortcut: char, callback: Callback) {
        if shortcut == '*'{
            self.any.push(callback);
        }else if self.store.contains_key(&shortcut){
            if let Some(callbacks) = self.store.get_mut(&shortcut){
                callbacks.push(callback);
            }
        } else {
            self.store.insert(shortcut, vec![callback]);
        }
    }
    pub fn run(&self, input:&str) ->  Result<Option<&Vec<Callback>>,String> {
        let input = input.trim();
        // as trim the first char isn't blank
        let mut prefix:&str = " ";
        let mut value:&str;

        let a= input.split_whitespace().collect::<Vec<&str>>();
        if a.len() == 2{
            prefix = a[0];
            value = &a[1];
            Ok(self.store.get(&prefix.chars().next().unwrap()))
        } else if a.len() == 1{
            Ok(Some(self.any.as_ref()))
        }else {
            Err(format!("Unknown command: {}", input))
        }




    }
}

#[derive(Default)]
pub struct Core{
    extension_lt:Vec<Box<dyn Extension>>,

}


impl Core {

    pub fn get_builder() -> Self {
        Self::default()
    }



    pub fn add_extension(mut self, ext: Box<dyn Extension>) -> Self {
        self.extension_lt.push(ext);
        self
    }
}