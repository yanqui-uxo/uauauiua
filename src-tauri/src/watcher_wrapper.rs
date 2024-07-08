use std::path::PathBuf;

use notify::{
    recommended_watcher, Event, EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use tauri::AppHandle;

use crate::utility::load_main;

struct ReloadHandler {
    path: PathBuf,
    app_handle: AppHandle,
}
impl EventHandler for ReloadHandler {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        if let EventKind::Modify(_) = event.unwrap().kind {
            load_main(self.path.clone(), self.app_handle.clone());
        };
    }
}
pub struct WatcherWrapper {
    app_handle: AppHandle,
    watcher: Option<RecommendedWatcher>,
}

impl WatcherWrapper {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            watcher: None,
        }
    }

    pub fn watch(&mut self, path: PathBuf) {
        let mut watcher = recommended_watcher(ReloadHandler {
            path: path.clone(),
            app_handle: self.app_handle.clone(),
        })
        .unwrap();
        watcher.watch(&path, RecursiveMode::Recursive).unwrap();
        self.watcher = Some(watcher);
    }
}
