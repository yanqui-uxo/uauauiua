use std::{
    collections::{HashMap, HashSet},
    iter::Peekable,
    sync::{
        LazyLock,
        mpsc::{Receiver, SendError, Sender, channel},
    },
    time::Duration,
};

use crossterm::event::KeyCode;
use rodio::{Source, buffer::SamplesBuffer, source::Repeat};
use uiua::{NativeSys, SysBackend};

pub const CHANNEL_NUM: u16 = 2;

pub fn new_mixer(is_recording: bool) -> (MixerController, Mixer) {
    let (event_tx, event_rx) = channel();
    let (recording_tx, recording_rx) = channel();
    (
        MixerController::new(is_recording, event_tx, recording_rx),
        Mixer::new(is_recording, event_rx, recording_tx),
    )
}

pub static SAMPLE_RATE: LazyLock<u32> = LazyLock::new(|| NativeSys.audio_sample_rate());

pub enum MixerCommand {
    Source(SamplesBuffer<f32>),
    ToggleHold(KeyCode, SamplesBuffer<f32>),
    Start,
    StopPlayback,
    StopRecording,
}

pub struct MixerController {
    is_recording: bool,
    command_tx: Sender<MixerCommand>,
    recording_rx: Receiver<f32>,
    held_sources: HashSet<KeyCode>,
}

impl MixerController {
    fn new(
        is_recording: bool,
        event_tx: Sender<MixerCommand>,
        recording_rx: Receiver<f32>,
    ) -> Self {
        MixerController {
            command_tx: event_tx,
            recording_rx,
            held_sources: HashSet::default(),
            is_recording,
        }
    }
    pub fn add(&self, source: SamplesBuffer<f32>) -> Result<(), SendError<MixerCommand>> {
        self.command_tx.send(MixerCommand::Source(source))
    }
    pub fn toggle_hold(
        &mut self,
        key: KeyCode,
        source: SamplesBuffer<f32>,
    ) -> Result<(), SendError<MixerCommand>> {
        self.command_tx
            .send(MixerCommand::ToggleHold(key, source))?;
        if self.held_sources.contains(&key) {
            self.held_sources.remove(&key);
        } else {
            self.held_sources.insert(key);
        }
        Ok(())
    }

    pub fn start_recording(&mut self) -> Result<(), SendError<MixerCommand>> {
        self.command_tx.send(MixerCommand::Start)?;
        self.is_recording = true;
        Ok(())
    }
    pub fn stop_playback(&mut self) -> Result<(), SendError<MixerCommand>> {
        self.command_tx.send(MixerCommand::StopPlayback)?;
        self.held_sources.clear();
        Ok(())
    }

    pub fn get_recording(&mut self) -> Vec<f32> {
        self.recording_rx.try_iter().collect()
    }

    pub fn stop_recording(&mut self) -> Result<Vec<f32>, SendError<MixerCommand>> {
        self.command_tx.send(MixerCommand::StopRecording)?;
        self.is_recording = false;
        Ok(self.get_recording())
    }

    pub fn held_sources(&self) -> &HashSet<KeyCode> {
        &self.held_sources
    }
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }
}

pub struct Mixer {
    command_rx: Receiver<MixerCommand>,
    regular_sources: Vec<Peekable<SamplesBuffer<f32>>>,
    held_sources: HashMap<KeyCode, Peekable<Repeat<SamplesBuffer<f32>>>>,
    is_recording: bool,
    recording_tx: Sender<f32>,
}

impl Mixer {
    fn new(
        is_recording: bool,
        event_rx: Receiver<MixerCommand>,
        recording_tx: Sender<f32>,
    ) -> Self {
        Mixer {
            command_rx: event_rx,
            regular_sources: Vec::default(),
            held_sources: HashMap::default(),
            is_recording,
            recording_tx,
        }
    }

    fn handle_events(&mut self) {
        self.command_rx.try_iter().for_each(|e| match e {
            MixerCommand::Source(s) => {
                self.regular_sources.push(s.peekable());
            }
            MixerCommand::ToggleHold(k, s) =>
            {
                #[allow(clippy::map_entry)]
                if self.held_sources.contains_key(&k) {
                    self.held_sources.remove(&k);
                } else {
                    self.held_sources.insert(k, s.repeat_infinite().peekable());
                }
            }
            MixerCommand::Start => {
                self.is_recording = true;
            }
            MixerCommand::StopPlayback => {
                self.regular_sources.clear();
                self.held_sources.clear();
                self.regular_sources.shrink_to_fit();
                self.held_sources.shrink_to_fit();
            }
            MixerCommand::StopRecording => {
                self.is_recording = false;
            }
        });
    }
}

impl Iterator for Mixer {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.handle_events();

        self.regular_sources.retain_mut(|x| x.peek().is_some());
        self.held_sources.retain(|_, v| v.peek().is_some());

        let sample = self
            .regular_sources
            .iter_mut()
            .map(|s| {
                s.next()
                    .expect("Empty non-held sources should have been removed")
            })
            .chain(self.held_sources.values_mut().map(|s| {
                s.next()
                    .expect("Empty held sources should have been removed")
            }))
            .sum::<f32>()
            .clamp(-1.0, 1.0);

        if self.is_recording {
            self.recording_tx.send(sample).unwrap();
        }

        Some(sample)
    }
}

impl Source for Mixer {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        CHANNEL_NUM
    }

    fn sample_rate(&self) -> u32 {
        *SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
