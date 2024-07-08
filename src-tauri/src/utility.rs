use std::fs::File;
use std::path::PathBuf;

use tauri::api::dialog::FileDialogBuilder;
use uiua::{NativeSys, SysBackend};

use crate::uiua_wrapper::{AudioData, UiuaWrapper};

pub const MAIN_FILE_NAME: &str = "main.ua";
pub const PRELUDE_FILE_NAME: &str = "prelude.ua";

#[taurpc::procedures(export_to = "../src/types.ts")]
pub trait Api {
    #[taurpc(event)]
    async fn file_loaded(clips: AudioData);

    #[taurpc(event)]
    async fn error(error: String);
}

#[derive(Clone)]
pub struct ApiImpl;

#[taurpc::resolvers]
impl Api for ApiImpl {}

pub fn pick_folder<F: FnMut(PathBuf) + Send + 'static>(mut f: F) {
    FileDialogBuilder::new().pick_folder(move |path_opt| {
        if let Some(path) = path_opt {
            f(path);
        }
    });
}

pub fn load_main(mut path: PathBuf, app_handle: tauri::AppHandle) {
    NativeSys.change_directory(path.to_str().unwrap()).unwrap();
    path.push(MAIN_FILE_NAME);
    let code = std::fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .unwrap();

    let mut uiua = UiuaWrapper::new();

    match uiua.run_str(&code) {
        Ok(data) => TauRpcApiEventTrigger::new(app_handle)
            .file_loaded(data)
            .unwrap(),
        Err(e) => TauRpcApiEventTrigger::new(app_handle).error(e).unwrap(),
    };
}

pub fn create_file(path: &PathBuf) -> File {
    std::fs::File::create(path)
        .unwrap_or_else(|_| panic!("Could not create file at {}", path.display()))
}
