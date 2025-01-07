mod recording;
mod uauauiua;

use recording::new_mixer;
use uauauiua::{Uauauiua, MAIN_PATH};

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use crossterm::event::{Event as CrosstermEventKind, KeyEventKind};
use notify::{recommended_watcher, Watcher};
use rodio::{OutputStream, Sink};

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let (mixer_controller, mixer) = new_mixer();
    sink.append(mixer);

    let uauauiua = Arc::new(Mutex::new(Uauauiua::new()));
    let uauauiua2 = Arc::clone(&uauauiua);

    // TODO: check for path validity
    let mut watcher = recommended_watcher(move |e: notify::Result<notify::Event>| {
        if let notify::EventKind::Modify(_) = e.unwrap().kind {
            *uauauiua2.lock().unwrap() = Uauauiua::new();
        }
    })
    .unwrap();
    watcher
        .watch(Path::new(MAIN_PATH), notify::RecursiveMode::NonRecursive)
        .unwrap();

    loop {
        if let CrosstermEventKind::Key(e) = crossterm::event::read().unwrap() {
            if let KeyEventKind::Press = e.kind {
                if let Ok(ref mut uauauiua) = *uauauiua.lock().unwrap() {
                    if let Some(s) = uauauiua.key_sources().get(&e.code) {
                        mixer_controller.add(s.clone()).unwrap();
                    }
                }
            }
        }
    }
}
