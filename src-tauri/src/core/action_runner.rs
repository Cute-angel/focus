use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

pub type Action = Box<dyn Fn(String, AppHandle) + Send + Sync + 'static>;
type ArcMutex<T> = Arc<Mutex<T>>;

pub struct ActionRunner {
    val: HashMap<String, Action>,
}

impl Default for ActionRunner {
    fn default() -> Self {
        Self {
            val: HashMap::new(),
        }
    }
}

impl ActionRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, key: &str, f: Action) {
        self.val.insert(key.to_string(), Box::new(f));
    }

    pub fn get(&self, key: &str) -> Option<&Action> {
        self.val.get(key)
    }
}
