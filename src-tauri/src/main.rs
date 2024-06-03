// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::needless_pass_by_value)]

mod audio;
mod uiua_wrapper;

use std::sync::{Arc, Mutex};

use audio::{AudioApi, AudioApiImpl};
use taurpc::Router;
use uiua_wrapper::UiuaWrapper;

#[taurpc::procedures(export_to = "../src/types.ts")]
trait Api {
    async fn run_code(code: String) -> Result<(), String>;
}

#[derive(Clone)]
struct ApiImpl {
    uiua: Arc<Mutex<UiuaWrapper>>,
}

#[taurpc::resolvers]
impl Api for ApiImpl {
    async fn run_code(self, code: String) -> Result<(), String> {
        self.uiua.lock().unwrap().run_str(&code)
    }
}

#[tokio::main]
async fn main() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();

    let player = Arc::new(Mutex::new(audio::Player::new(&handle)));
    let uiua = Arc::new(Mutex::new(UiuaWrapper::new()));
    let base_impl = ApiImpl {
        uiua: Arc::clone(&uiua),
    };
    let audio_impl = AudioApiImpl { player, uiua };

    let router = Router::new()
        .merge(base_impl.into_handler())
        .merge(audio_impl.into_handler());

    tauri::Builder::default()
        .invoke_handler(router.into_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
