// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::needless_pass_by_value)]

mod uiua_wrapper;

use std::sync::{Arc, Mutex};

use uiua::{
    format::{format_str, FormatConfig},
    NativeSys, SysBackend,
};

use uiua_wrapper::{AudioData, UiuaWrapper};

#[taurpc::procedures(export_to = "../src/types.ts")]
trait Api {
    async fn format_code(code: String) -> Result<String, String>;
    async fn run_code(code: String) -> Result<AudioData, String>;
    async fn sample_rate() -> u32;
}

#[derive(Clone)]
struct ApiImpl {
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

    async fn run_code(self, code: String) -> Result<AudioData, String> {
        self.uiua.lock().unwrap().run_str(&code)
    }

    async fn sample_rate(self) -> u32 {
        NativeSys.audio_sample_rate()
    }
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(taurpc::create_ipc_handler(
            ApiImpl {
                uiua: Arc::new(Mutex::new(UiuaWrapper::new())),
            }
            .into_handler(),
        ))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
