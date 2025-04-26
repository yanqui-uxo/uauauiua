use std::{fs, mem};

use crate::recording::{CHANNEL_NUM, MixerController, SAMPLE_RATE, new_mixer};
use crate::uiua_extension::UiuaExtension;

use anyhow::{anyhow, ensure};
use crossterm::event::KeyCode;
use hound::{SampleFormat, WavSpec, WavWriter};
use indexmap::IndexSet;
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
    fn new(is_recording_main: bool, is_recording_secondary: bool) -> Self {
        let (stream, stream_handle) =
            OutputStream::try_default().expect("should have initialized audio output stream");
        let (mixer_controller, mixer) = new_mixer(is_recording_main, is_recording_secondary);
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
    partial_main_recording: Vec<f32>,
    partial_secondary_recording: Vec<f32>,
    audio_handler: AudioHandler,
}

impl Default for Uauauiua {
    fn default() -> Self {
        Uauauiua {
            uiua_extension: UiuaExtension::default(),
            partial_main_recording: Vec::default(),
            partial_secondary_recording: Vec::default(),
            audio_handler: AudioHandler::new(false, false),
        }
    }
}

impl Uauauiua {
    pub fn load(&mut self) -> anyhow::Result<()> {
        self.uiua_extension.load()
    }

    fn mixer_controller(&self) -> &MixerController {
        self.audio_handler.mixer_controller()
    }
    fn mixer_controller_mut(&mut self) -> &mut MixerController {
        self.audio_handler.mixer_controller_mut()
    }

    pub fn reinit_audio(&mut self) {
        let mut main_recording = self.mixer_controller_mut().get_main_recording();
        self.partial_main_recording.append(&mut main_recording);
        let mut secondary_recording = self.mixer_controller_mut().get_secondary_recording();
        self.partial_secondary_recording
            .append(&mut secondary_recording);

        self.audio_handler = AudioHandler::new(
            self.mixer_controller().is_recording_main(),
            self.mixer_controller().is_recording_secondary(),
        );
    }

    pub fn start_main_recording(&mut self) -> anyhow::Result<()> {
        self.mixer_controller_mut()
            .start_main_recording()
            .map_err(|_| anyhow!("could not start main recording"))
    }
    pub fn start_secondary_recording(&mut self) -> anyhow::Result<()> {
        self.mixer_controller_mut()
            .start_secondary_recording()
            .map_err(|_| anyhow!("could not start secondary recording"))
    }

    pub fn stop_playback(&mut self) -> anyhow::Result<()> {
        self.mixer_controller_mut()
            .stop_playback()
            .map_err(|_| anyhow!("could not stop playback"))
    }

    pub fn stop_main_recording_and_playback(&mut self) -> anyhow::Result<Vec<f32>> {
        let ret = self
            .mixer_controller_mut()
            .stop_main_recording()
            .map_err(|_| anyhow!("could not stop main recording"));
        self.stop_playback()?;
        ret
    }

    pub fn stop_secondary_recording_and_playback(&mut self) -> anyhow::Result<Vec<f32>> {
        let ret = self
            .mixer_controller_mut()
            .stop_secondary_recording()
            .map_err(|_| anyhow!("could not stop secondary recording"));
        self.stop_playback()?;
        ret
    }

    pub fn add_to_mixer(&mut self, key: KeyCode, toggle_hold: bool) -> anyhow::Result<()> {
        let source = self
            .uiua_extension
            .key_sources()
            .get(&key)
            .ok_or(anyhow!("key {key} not recognized"))?;

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

    pub fn clear_stack(&mut self) {
        self.uiua_extension.clear_stack();
    }

    pub fn clear_recordings(&mut self) {
        self.uiua_extension.clear_new_values();
    }

    pub fn save_main_recording(&mut self, recording: &[f32], name: &str) -> anyhow::Result<()> {
        if name.is_empty() {
            return Ok(());
        }

        let mut recording_iter = mem::take(&mut self.partial_main_recording)
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

    pub fn save_secondary_recording(&mut self, recording: &[f32], name: &str) {
        if name.is_empty() {
            return;
        }

        let recording: Vec<f32> = mem::take(&mut self.partial_secondary_recording)
            .into_iter()
            .chain(recording.iter().copied())
            .collect();
        let len = recording.len();
        let mut recording_value: Value = recording.into_iter().map(f64::from).collect();
        *recording_value.shape_mut() = [len / CHANNEL_NUM as usize, CHANNEL_NUM as usize].into();

        self.uiua_extension.add_value(name, recording_value);
    }

    pub fn defined_sources(&self) -> IndexSet<KeyCode> {
        self.uiua_extension.key_sources().keys().copied().collect()
    }

    pub fn held_sources(&self) -> &IndexSet<KeyCode> {
        self.mixer_controller().held_sources()
    }

    pub fn secondary_recording_names(&self) -> IndexSet<String> {
        self.uiua_extension.new_value_names()
    }

    pub fn stack(&self) -> &[Value] {
        self.uiua_extension.stack()
    }

    pub fn is_recording_main(&self) -> bool {
        self.mixer_controller().is_recording_main()
    }
    pub fn is_recording_secondary(&self) -> bool {
        self.mixer_controller().is_recording_secondary()
    }
}
