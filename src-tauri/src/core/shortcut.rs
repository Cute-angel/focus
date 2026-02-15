use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::ops::Deref;
// use to sort result


pub enum ShortcutKey {
    Key(String),
    Fixed(u64),
    Any(fn(String)->u64)
}



pub struct Shortcut {
    target:ShortcutKey

}


/// match rule:
/// accurate match > any 
pub struct ShortcutsDispatcher  {
    keyword_type_shortcut: HashMap<String, Shortcut>,
    any_type_shortcut: Vec<Shortcut>,
    fixed_type_shortcut: Vec<Shortcut>,

}

pub enum ShortCutError{
    SameKeyWord
}
impl ShortcutsDispatcher {
    pub fn run(&self, input:&str) {
        let lt  = input.trim().split_whitespace().collect::<Vec<_>>();
        // key type
        if let Some(prefix )= lt.first() {
            if let Some(_) =  self.keyword_type_shortcut.get(*prefix) {
                
            }
        }
        let rest_part = lt.iter().skip(1).collect::<Vec<String>>().join(" ");
        self.run_any_shortcut(&rest_part);
        self.run_fixed_shortcut();
        
    }
    
    pub fn add_shortcut(&mut self, shortcut: Shortcut) -> Result<(),ShortCutError> {
        match &shortcut.target {
            ShortcutKey::Key(k) => {
               if self.keyword_type_shortcut.insert(k.clone(), shortcut).is_none() {
                    Ok(())
                } else {
                    Err(ShortCutError::SameKeyWord)
                }
            }
            ShortcutKey::Fixed(_) => {
                self.fixed_type_shortcut.push(shortcut);
                Ok(())
            }
            ShortcutKey::Any(_) => {
                self.any_type_shortcut.push(shortcut);
                Ok(())
            }
        }
    }
    
    fn run_any_shortcut(&self, text:&str)  {
        todo!()
    }
    
    fn run_fixed_shortcut(&self, shortcut: Shortcut) -> Result<(),ShortCutError> {
        todo!()
    }
}