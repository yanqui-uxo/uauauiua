use std::collections::HashSet;

use crate::recording::{CHANNEL_NUM, MixerController, SAMPLE_RATE, new_mixer};
use crate::uiua_extension::UiuaExtension;

use anyhow::{anyhow, ensure};
use crossterm::event::KeyCode;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use uiua::Value;

pub struct Uauauiua {
    uiua_extension: UiuaExtension,
    mixer_controller: MixerController,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    _sink: Sink,
}

// TODO: use tracing to properly handle stream errors
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

    pub fn load(&mut self) -> anyhow::Result<Vec<Value>> {
        self.uiua_extension.load()
    }

    pub fn start_recording(&self) {
        self.mixer_controller.start_recording();
    }

    pub fn stop_playback(&mut self) {
        self.mixer_controller.stop_playback();
    }
    pub fn stop_recording_and_playback(&mut self) -> Vec<f32> {
        let ret = self.mixer_controller.stop_recording();
        self.mixer_controller.stop_playback();
        ret
    }

    pub fn add_to_mixer(&mut self, key: KeyCode, toggle_hold: bool) -> anyhow::Result<()> {
        let source = self
            .uiua_extension
            .key_sources()
            .get(&key)
            .ok_or(anyhow!("did not recognize key {key}"))?;

        ensure!(
            source.channels() == CHANNEL_NUM,
            "incorrect number of channels; expected {CHANNEL_NUM}"
        );
        ensure!(
            source.sample_rate() == *SAMPLE_RATE,
            "incorrect sample rate; expected {}",
            *SAMPLE_RATE
        );

        if toggle_hold {
            self.mixer_controller.toggle_hold(key, source.clone());
        } else {
            self.mixer_controller.add(source.clone());
        }

        Ok(())
    }

    pub fn held_sources(&self) -> &HashSet<KeyCode> {
        self.mixer_controller.held_sources()
    }
}
