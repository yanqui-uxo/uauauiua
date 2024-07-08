// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::needless_pass_by_value)]

mod uiua_wrapper;
mod utility;
mod watcher_wrapper;

use std::sync::Mutex;

use tauri::{CustomMenuItem, Manager, Menu, MenuEntry};
use taurpc::create_ipc_handler;
use utility::{
    create_file, load_main, pick_folder, Api, ApiImpl, MAIN_FILE_NAME, PRELUDE_FILE_NAME,
};
use watcher_wrapper::WatcherWrapper;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(create_ipc_handler(ApiImpl.into_handler()))
        .setup(|app| {
            app.manage(Mutex::new(WatcherWrapper::new(app.handle())));
            Ok(())
        })
        .menu(Menu::with_items([
            MenuEntry::CustomItem(CustomMenuItem::new("new", "New")),
            MenuEntry::CustomItem(CustomMenuItem::new("open", "Open")),
        ]))
        .on_menu_event(|e| {
            let app_handle = e.window().app_handle();
            match e.menu_item_id() {
                "new" => {
                    pick_folder(move |path| {
                        create_file(&path.join(MAIN_FILE_NAME));

                        let prelude_path = path.join(PRELUDE_FILE_NAME);
                        create_file(&prelude_path);
                        std::fs::write(
                            path.join(PRELUDE_FILE_NAME),
                            include_str!("../../prelude.ua"),
                        )
                        .unwrap();

                        app_handle
                            .state::<Mutex<WatcherWrapper>>()
                            .lock()
                            .unwrap()
                            .watch(path);
                    });
                }
                "open" => pick_folder(move |path| {
                    load_main(path.clone(), e.window().app_handle());
                    app_handle
                        .state::<Mutex<WatcherWrapper>>()
                        .lock()
                        .unwrap()
                        .watch(path);
                }),
                other => panic!("Unexpected event id '{other}'"),
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
