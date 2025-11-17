use crate::api::action_runner::{Action, ActionRunner};
use crate::api::command_tree::{Callback, CommandContext, CommandDispatcher, CommandNode, StringArgument};
use crate::api::extension::{action, Extension, ExtensionResult, Results};
use crate::utils::{to_base64, IconExtractor};
use lnk_parser::LNKParser;
use pinyin::ToPinyin;
use rust_fuzzy_search::fuzzy_search_best_n;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use walkdir::WalkDir;

static SEARCH_TABLE:LazyLock<Arc<Mutex<HashMap<String,Program>>>> = LazyLock::new(
    || Arc::new(Mutex::new(HashMap::new()))
);


pub struct Launcher {
    data_puf:PathBuf,
    recursive_depth: usize,

}

#[derive(Debug ,Clone)]
#[derive(Eq, Hash)]
struct Program{
    pub display_name:String,
    pub path:String,
}

impl PartialEq for Program {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}


impl Program {
    fn new(name:&str,path:&str) -> Program {
        Program{
            display_name:name.to_string(),
            path:path.to_string(),
        }
    }



    pub fn get_path(&self) -> &str{
        &self.path
    }

    pub fn get_display_name(&self) -> &str{
        &self.display_name
    }
}


impl Default for Launcher {
    fn default() -> Launcher {
        Launcher {
            data_puf:PathBuf::new(),
            recursive_depth: 4,
        }
    }
}
impl Launcher {
    pub fn init(&self) {
        self.build_index();
    }

    fn search_program_keys(&self,num:usize,input:&str) -> Vec<Program> {
        //standardization
        let input = input.to_lowercase();


        let lt = SEARCH_TABLE.lock().unwrap().keys().cloned().collect::<Vec<String>>();
        let refs: Vec<&str> = lt.iter().map(|s| s.as_str()).collect();
        let fuzzy_str =  fuzzy_search_best_n(&input,refs.as_ref(),num).into_iter().map(|(key,_)|key).collect::<Vec<&str>>();

        let mut map:HashSet<Program> = HashSet::new();
        let mut res:Vec<Program> = Vec::with_capacity(fuzzy_str.len());


        let lock_data = SEARCH_TABLE.lock().unwrap();
        for i in fuzzy_str.iter() {
            if let Some(dat) = lock_data.get(*i) {
                if map.insert(dat.clone()){
                    res.push(dat.clone());
                }
            }
        };
        res
    }

    pub fn create_watcher(&self) {
        let th = thread::spawn(move || {

        });
    }



    fn contains_chinese(&self, s: &str) -> bool {
        fn is_chinese(c: char) -> bool {
            (c >= '\u{4e00}' && c <= '\u{9fff}')      // CJK Unified Ideographs
                || (c >= '\u{3400}' && c <= '\u{4dbf}')  // CJK Extension A
                || (c >= '\u{20000}' && c <= '\u{2a6df}') // Extension B
                || (c >= '\u{2a700}' && c <= '\u{2b73f}') // Extension C
                || (c >= '\u{2b740}' && c <= '\u{2b81f}') // Extension D
                || (c >= '\u{2b820}' && c <= '\u{2ceaf}') // Extension E
        }
        s.chars().any(is_chinese)
    }


    fn start_menu_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(programdata) = std::env::var_os("PROGRAMDATA") {
            paths.push(PathBuf::from(programdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs"));
        }

        if let Some(appdata) = std::env::var_os("APPDATA") {
            paths.push(PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs"));
        }
        paths
    }

    fn resolve_lnk(&self, path: &PathBuf) -> Option<String> {
        match  LNKParser::from_path(path.to_str()?) {
            Ok(lnk) => {
                lnk.get_target_full_path().clone()
            }
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        }
    }

    fn build_index(&self){



        let mut search:Vec<(Vec<String>,String,String)> = Vec::with_capacity(100);
        let mut data:Vec<(String,String)> = Vec::with_capacity(100);

        for start_menu in self.start_menu_paths() {
            dbg!(&start_menu);
            if start_menu.exists() {
                for entry in WalkDir::new(start_menu)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map(|s| s.eq_ignore_ascii_case("lnk")).unwrap_or(false))
                {
                    let path = entry.path().to_path_buf();
                    let mut file_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                    if let Some(input) = file_name.strip_prefix("MY_COMPUTER\\") {
                        file_name = input.to_string();
                    }


                    if let Some(real_path) =  self.resolve_lnk(&path){
                        let _ = &search.push(
                            (self.build_key_from_name(file_name.clone()),real_path.clone(),file_name)
                        );
                    }

                }
            }
        }


        let mut lock_search = SEARCH_TABLE.lock().unwrap();
        for (keys,value,Display) in search{
            for key in keys {
                lock_search.insert(
                    key,
                    Program::new(
                        Display.as_str(),
                        value.as_str(),
                    )
                );
            }
        }
        drop(lock_search);


    }

    fn build_key_from_name(&self, file_name:String) -> Vec<String> {
        let mut keys = Vec::new();
        if self.contains_chinese(&file_name) {
            keys.push(
                file_name.as_str().to_pinyin().flatten().map(|x| {x.plain()}).collect::<Vec<&str>>().join(" ")
            );
            keys.push(
                file_name.as_str().to_pinyin().flatten().map(|x| {x.first_letter()}).collect::<Vec<&str>>().join("")
            )
        }else {


            for i in file_name.split_whitespace() {
                keys.push(
                    i.to_lowercase()
                )
            }
        }
        keys
    }

    fn get_action(&self) -> Action {
        let f = |val: String, app: AppHandle| {
            let r = match app.opener().open_path(val, None::<&str>) {
                Ok(_) => {}
                Err(_) => {}
            };
        };
        Box::new(f)
    }

    fn get_callback(&self) -> Callback {
        let callback  = move |ctx:CommandContext,app:AppHandle |{
            if let Some(input) = ctx.get_parm("app_query"){

                let launch = Launcher::default();

                let test = dbg!(launch.search_program_keys(20,input));

                let plugin_res = test.iter().map(
                    |item|{
                        let mut icon = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h15A2.25 2.25 0 0 1 21.75 12v.75m-8.69-6.44-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z" />
                                    </svg>"#.to_string();
                        if Path::new(item.get_path()).exists() {
                            if let Some(data) = IconExtractor::default().get_icon(&PathBuf::from(item.get_path())){
                                icon = format!("<img src=\"data:image/png;base64,{}\" alt=\"Image\" />", to_base64(data))
                            }
                        }

                        ExtensionResult{
                            icon,
                            title: item.display_name.clone(),
                            description: item.path.clone(),
                            actions: vec![
                                action{
                                    icon: "hide".to_string(),
                                    tooltip: "".to_string(),
                                    value: item.path.to_string(),
                                    id: "launcher".to_string(),
                                }
                            ],
                        }
                    }
                ).collect::<Vec<ExtensionResult>>();

                let res = Results{
                    total_count: plugin_res.len(),
                    items: plugin_res,
                };
                Box::new(res) as Box<dyn Any + Send + 'static>
            }else {
                Box::new(()) as Box<dyn Any + Send + 'static>
            }

        };
        Box::new(move |ctx, app| -> Box<dyn Any> {
            callback(ctx, app)
        })



    }

    pub fn get_node(&self) -> CommandNode{
        let cmd = CommandNode::new("app").then(
            CommandNode::new("app_query")
                .argument(StringArgument)
                .execute(self.get_callback())
        );
        cmd
    }

}


impl Extension for Launcher {
    fn OnMount(&self, command_dispatcher: &mut CommandDispatcher) {
        command_dispatcher.register(
            self.get_node()
        );

        let action_runner = ActionRunner::get_instance();
        action_runner.lock().unwrap().add("launcher",self.get_action());
    }

    fn OnUnmount(&self, command_dispatcher: &mut CommandDispatcher) {
        todo!()
    }
}