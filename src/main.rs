mod uauauiua;
use uauauiua::{Uauauiua, MAIN_PATH};

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, ensure};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};
use notify::{recommended_watcher, Watcher};
use rodio::{
    buffer::SamplesBuffer,
    dynamic_mixer::{DynamicMixer, DynamicMixerController},
    OutputStream, Sink,
};
use uiua::{NativeSys, SysBackend, Uiua};

// TODO: make custom mixer? This mixer seems to make things crunchy
fn new_mixer(sink: &Sink) -> Arc<DynamicMixerController<f32>> {
    let (mixer_controller, mixer) = rodio::dynamic_mixer::mixer(2, NativeSys.audio_sample_rate());
    sink.append(mixer);
    mixer_controller
}

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut mixer_controller = new_mixer(&sink);

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
        if let CrosstermEvent::Key(e) = crossterm::event::read().unwrap() {
            if let KeyEventKind::Press = e.kind {
                if let Ok(ref mut uauauiua) = *uauauiua.lock().unwrap() {
                    if let Some(s) = uauauiua.key_sources().get(&e.code) {
                        if sink.empty() {
                            mixer_controller = new_mixer(&sink);
                        }
                        mixer_controller.add(s.clone());
                    }
                }
            }
        }
    }
}
