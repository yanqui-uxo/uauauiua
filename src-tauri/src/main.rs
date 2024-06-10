// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::needless_pass_by_value)]

mod audio;
mod uiua_wrapper;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use uiua::format::{format_str, FormatConfig};

use uiua_wrapper::UiuaWrapper;

#[taurpc::procedures(export_to = "../src/types.ts")]
trait Api {
    async fn format_code(code: String) -> Result<String, String>;
    async fn run_code(code: String) -> Result<(), String>;

    async fn var_samples() -> HashMap<String, Vec<f32>>;
    async fn load_stack_sample() -> ();
    async fn load_var_sample(var: String) -> ();
    async fn play() -> ();
}

#[derive(Clone)]
struct ApiImpl {
    player: Arc<Mutex<audio::Player>>,
    uiua: Arc<Mutex<UiuaWrapper>>,
}

#[taurpc::resolvers]
impl Api for ApiImpl {
    async fn format_code(self, code: String) -> Result<String, String> {
        match format_str(&code, &FormatConfig::default().with_trailing_newline(false)) {
            Ok(out) => Ok(out.output),
            Err(e) => Err(e.message()),
        }
    }

    async fn run_code(self, code: String) -> Result<(), String> {
        self.player
            .lock()
            .unwrap()
            .load_from_code(code, &mut self.uiua.lock().unwrap())
    }

    async fn var_samples(self) -> HashMap<String, Vec<f32>> {
        self.uiua.lock().unwrap().bound_samples()
    }
    async fn load_stack_sample(self) {
        self.player.lock().unwrap().load_stack_sample();
    }
    async fn load_var_sample(self, var: String) {
        self.player.lock().unwrap().load_var_sample(var);
    }
    async fn play(self) {
        self.player
            .lock()
            .unwrap()
            .handle_action(audio::PlayerAction::Play);
    }
}

#[tokio::main]
async fn main() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();

    tauri::Builder::default()
        .invoke_handler(taurpc::create_ipc_handler(
            ApiImpl {
                player: Arc::new(Mutex::new(audio::Player::new(&handle))),
                uiua: Arc::new(Mutex::new(UiuaWrapper::new())),
            }
            .into_handler(),
        ))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
