use std::collections::HashSet;
use std::{fs, mem};

use crate::recording::{CHANNEL_NUM, MixerController, SAMPLE_RATE, new_mixer};
use crate::uiua_extension::UiuaExtension;

use anyhow::{anyhow, ensure};
use crossterm::event::KeyCode;
use hound::{SampleFormat, WavSpec, WavWriter};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use uiua::Value;

const RECORDINGS_DIR: &str = "recordings";

struct AudioHandler {
    mixer_controller: MixerController,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    _sink: Sink,
}
impl AudioHandler {
    fn new(is_recording: bool) -> Self {
        let (stream, stream_handle) =
            OutputStream::try_default().expect("should have initialized audio output stream");
        let (mixer_controller, mixer) = new_mixer(is_recording);
        let sink = Sink::try_new(&stream_handle).expect("should have initialized audio sink");
        sink.append(mixer);

        Self {
            mixer_controller,
            _stream: stream,
            _stream_handle: stream_handle,
            _sink: sink,
        }
    }

    fn mixer_controller(&self) -> &MixerController {
        &self.mixer_controller
    }
    fn mixer_controller_mut(&mut self) -> &mut MixerController {
        &mut self.mixer_controller
    }
}

pub struct Uauauiua {
    uiua_extension: UiuaExtension,
    partial_recording: Vec<f32>,
    audio_handler: AudioHandler,
}

impl Default for Uauauiua {
    fn default() -> Self {
        Uauauiua {
            uiua_extension: UiuaExtension::default(),
            partial_recording: Vec::default(),
            audio_handler: AudioHandler::new(false),
        }
    }
}

impl Uauauiua {
    pub fn load(&mut self) -> anyhow::Result<()> {
        self.uiua_extension.load()
    }

    pub fn reinit_audio(&mut self) {
        let held_sources = self.mixer_controller().held_sources().clone();

        let mut recording = self.mixer_controller_mut().get_recording();
        self.partial_recording.append(&mut recording);

        self.audio_handler = AudioHandler::new(self.mixer_controller().is_recording());
        for key in held_sources {
            self.add_to_mixer(key, true)
                .expect("could not re-add held sources");
        }
    }

    fn mixer_controller(&self) -> &MixerController {
        self.audio_handler.mixer_controller()
    }
    fn mixer_controller_mut(&mut self) -> &mut MixerController {
        self.audio_handler.mixer_controller_mut()
    }

    pub fn start_recording(&mut self) -> anyhow::Result<()> {
        self.mixer_controller_mut()
            .start_recording()
            .map_err(|_| anyhow!("could not start recording"))
    }

    pub fn stop_playback(&mut self) -> anyhow::Result<()> {
        self.mixer_controller_mut()
            .stop_playback()
            .map_err(|_| anyhow!("could not stop playback"))
    }

    pub fn stop_recording_and_playback(&mut self) -> anyhow::Result<Vec<f32>> {
        let ret = self
            .mixer_controller_mut()
            .stop_recording()
            .map_err(|_| anyhow!("could not stop recording"));
        self.stop_playback()?;
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

        let source = source.clone();
        if toggle_hold {
            self.mixer_controller_mut()
                .toggle_hold(key, source)
                .map_err(|_| anyhow!("could not toggle hold for key {key}"))
        } else {
            self.mixer_controller_mut()
                .add(source)
                .map_err(|_| anyhow!("could not play audio for key {key}"))
        }
    }

    pub fn held_sources(&self) -> &HashSet<KeyCode> {
        self.mixer_controller().held_sources()
    }

    pub fn stack(&self) -> &[Value] {
        self.uiua_extension.stack()
    }

    pub fn save_recording(&mut self, recording: &[f32], name: &str) -> anyhow::Result<()> {
        if name.is_empty() {
            return Ok(());
        }

        let mut recording_iter = mem::take(&mut self.partial_recording)
            .into_iter()
            .chain(recording.iter().copied());

        let spec = WavSpec {
            channels: CHANNEL_NUM,
            sample_rate: *SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let _ = fs::create_dir(RECORDINGS_DIR);

        let mut writer = WavWriter::create(format!("{RECORDINGS_DIR}/{name}.wav"), spec)?;

        recording_iter.try_for_each(|x| writer.write_sample(x))?;
        writer.finalize()?;

        Ok(())
    }
}
