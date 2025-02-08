use std::collections::HashMap;

use crate::recording::{new_mixer, MixerController};
use crate::uiua_extension::UiuaExtension;

use anyhow::anyhow;
use crossterm::event::KeyCode;
use rodio::{OutputStream, OutputStreamHandle, Sink};

pub struct Uauauiua {
    uiua_extension: UiuaExtension,
    mixer_controller: MixerController,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    _sink: Sink,
}

impl Uauauiua {
    pub fn new() -> Self {
        let (stream, stream_handle) =
            OutputStream::try_default().expect("should have initialized audio output stream");
        let (mixer_controller, mixer) = new_mixer();
        let sink = Sink::try_new(&stream_handle).expect("should have initialized audio sink");
        sink.append(mixer);

        Uauauiua {
            uiua_extension: UiuaExtension::default(),
            mixer_controller,
            _stream: stream,
            _stream_handle: stream_handle,
            _sink: sink,
        }
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        self.uiua_extension.load()
    }

    pub fn new_values(&mut self) -> &mut HashMap<String, uiua::Value> {
        &mut self.uiua_extension.new_values
    }

    pub fn start_recording(&mut self) {
        self.mixer_controller.start_recording();
    }

    pub fn stop_recording(&mut self) -> Vec<f32> {
        self.mixer_controller.stop_recording()
    }

    pub fn add_key_source_to_mixer(&mut self, key: KeyCode) -> anyhow::Result<()> {
        let source = self
            .uiua_extension
            .data()
            .key_sources()
            .get(&key)
            .ok_or(anyhow!("Did not recognize key {key}"))?;
        self.mixer_controller.add(source.clone())?;
        Ok(())
    }
}
