// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::Serialize;
use serde_json::Number;
use tauri::Runtime;
use std::{ sync::LazyLock};

use everything_rs::{Everything, EverythingRequestFlags, EverythingSort, EverythingError};
use crate::api::extension;
use crate::api::extension::Results;

static EverythingInstance: LazyLock<Everything> = LazyLock::new(Everything::new);

#[derive(Serialize)]
#[serde(tag = "type" ,content = "data")]
pub enum PluginResult {
    FileFinder {name:String, path:String , is_folder: bool},
    Program {name:String, path:String},
    cal(Number)
}


pub fn get_file_finder_result(text_input:String) -> Result<(usize,Vec<PluginResult>),Error> {
    let everything = &*EverythingInstance;
    everything.set_max_results(5);

    everything.set_request_flags(
        EverythingRequestFlags::FileName
            | EverythingRequestFlags::FullPathAndFileName
            | EverythingRequestFlags::Size
            | EverythingRequestFlags::DateModified,
    );

    everything.set_sort(EverythingSort::FileListFilenameAscending);

    everything.set_search(&text_input);

    everything.query()?;

    let num_results = everything.get_num_results();
    let mut  list:Vec<PluginResult> = vec![];

    if num_results  > 0 {
        for (i , result ) in everything.full_path_iter().flatten().enumerate() {
            list.push(PluginResult::FileFinder { 
                name: everything.get_result_file_name(i as u32).unwrap(), 
                path: result, 
                is_folder: everything.is_result_folder(i as u32) 
            });
            // println!("{}. {} size:{}", i + 1, result,everything.get_result_size(i as u32)?);
        }
    };
    everything.reset();
    
    Ok((num_results as usize,list))

}


// 创建在我们程序中可能发生的所有错误
#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  EverythingError(#[from] everything_rs::EverythingError),
}


impl serde::Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::ser::Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}


#[tauri::command]
pub async  fn query<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>,input_text:String) -> Result<Results, String> {
  let mut a = Results{
      total_count:1,
      items:Vec::new(),
  };
  Ok(a)
}

#[tauri::command]
pub fn run_action(id:String) {
  println!("{id}")
}