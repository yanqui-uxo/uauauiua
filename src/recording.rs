use std::{
    collections::{HashMap, HashSet},
    sync::{
        LazyLock,
        mpsc::{Receiver, Sender, channel},
    },
    time::Duration,
};

use crossterm::event::KeyCode;
use rodio::{Source, buffer::SamplesBuffer, source::Repeat};
use uiua::{NativeSys, SysBackend};

pub fn new_mixer() -> (MixerController, Mixer) {
    let (event_tx, event_rx) = channel();
    let (recording_tx, recording_rx) = channel();
    (
        MixerController::new(event_tx, recording_rx),
        Mixer::new(event_rx, recording_tx),
    )
}

pub const CHANNEL_NUM: u16 = 2;
pub static SAMPLE_RATE: LazyLock<u32> = LazyLock::new(|| NativeSys.audio_sample_rate());

enum MixerEvent {
    Source(SamplesBuffer<f32>),
    ToggleHold(KeyCode, SamplesBuffer<f32>),
    Start,
    StopPlayback,
    StopRecording,
}

pub struct MixerController {
    event_tx: Sender<MixerEvent>,
    recording_rx: Receiver<f32>,
    held_sources: HashSet<KeyCode>,
}

impl MixerController {
    fn new(event_tx: Sender<MixerEvent>, recording_rx: Receiver<f32>) -> Self {
        MixerController {
            event_tx,
            recording_rx,
            held_sources: HashSet::default(),
        }
    }

    pub fn add(&self, source: SamplesBuffer<f32>) {
        self.event_tx.send(MixerEvent::Source(source)).unwrap();
    }
    pub fn toggle_hold(&mut self, key: KeyCode, source: SamplesBuffer<f32>) {
        self.event_tx
            .send(MixerEvent::ToggleHold(key, source))
            .unwrap();
        if self.held_sources.contains(&key) {
            self.held_sources.remove(&key);
        } else {
            self.held_sources.insert(key);
        }
    }

    pub fn start_recording(&self) {
        self.event_tx.send(MixerEvent::Start).unwrap();
    }
    pub fn stop_playback(&mut self) {
        self.event_tx.send(MixerEvent::StopPlayback).unwrap();
        self.held_sources.clear();
    }
    pub fn stop_recording(&mut self) -> Vec<f32> {
        self.event_tx.send(MixerEvent::StopRecording).unwrap();
        self.recording_rx.try_iter().collect()
    }

    pub fn held_sources(&self) -> &HashSet<KeyCode> {
        &self.held_sources
    }
}

pub struct Mixer {
    event_rx: Receiver<MixerEvent>,
    regular_sources: Vec<SamplesBuffer<f32>>,
    held_sources: HashMap<KeyCode, Repeat<SamplesBuffer<f32>>>,
    is_recording: bool,
    recording_tx: Sender<f32>,
}

impl Mixer {
    fn new(event_rx: Receiver<MixerEvent>, recording_tx: Sender<f32>) -> Self {
        Mixer {
            event_rx,
            regular_sources: Vec::default(),
            held_sources: HashMap::default(),
            is_recording: false,
            recording_tx,
        }
    }

    fn handle_events(&mut self) {
        self.event_rx.try_iter().for_each(|e| match e {
            MixerEvent::Source(s) => {
                self.regular_sources.push(s);
            }
            MixerEvent::ToggleHold(k, s) =>
            {
                #[allow(clippy::map_entry)]
                if self.held_sources.contains_key(&k) {
                    self.held_sources.remove(&k);
                } else {
                    self.held_sources.insert(k, s.repeat_infinite());
                }
            }
            MixerEvent::Start => {
                self.is_recording = true;
            }
            MixerEvent::StopPlayback => {
                self.regular_sources.clear();
                self.held_sources.clear();
                self.regular_sources.shrink_to_fit();
                self.held_sources.shrink_to_fit();
            }
            MixerEvent::StopRecording => {
                self.is_recording = false;
            }
        });
    }
}

impl Iterator for Mixer {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.handle_events();

        let sample = self
            .regular_sources
            .iter_mut()
            .map(|s| s.next().unwrap_or_default())
            .chain(
                self.held_sources
                    .values_mut()
                    .map(|s| s.next().unwrap_or_default()),
            )
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
