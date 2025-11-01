use serde::Serialize;

#[derive(serde::Serialize ,Debug)]
pub struct action {
    icon:String,
    tooltip:String,
    value:String,
}

#[derive(serde::Serialize ,Debug)]
pub struct ExtensionResult {
    icon:String,
    title:String,
    description:String,
    actions: Vec<action>,
}

#[derive(serde::Serialize ,Debug)]
pub struct Results {
    pub(crate) total_count:usize,
    pub(crate) items:Vec<ExtensionResult>,
}

