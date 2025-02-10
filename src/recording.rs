use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        LazyLock,
    },
    time::Duration,
};

use anyhow::ensure;
use rodio::Source;
use uiua::{NativeSys, SysBackend};

type BoxedSource = Box<dyn Source<Item = f32> + Send>;

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
    Source(BoxedSource),
    Start,
    Stop,
}

pub struct MixerController {
    event_tx: Sender<MixerEvent>,
    recording_rx: Receiver<f32>,
    // TODO: do something with this field or remove it
    is_recording: bool,
}

impl MixerController {
    fn new(event_tx: Sender<MixerEvent>, recording_rx: Receiver<f32>) -> Self {
        MixerController {
            event_tx,
            recording_rx,
            is_recording: false,
        }
    }

    pub fn add<T>(&self, source: T) -> anyhow::Result<()>
    where
        T: Source<Item = f32> + Send + 'static,
    {
        ensure!(
            source.channels() == CHANNEL_NUM,
            format!("incorrect number of channels; expected {CHANNEL_NUM}")
        );
        ensure!(
            source.sample_rate() == *SAMPLE_RATE,
            format!("incorrect sample rate; expected {}", *SAMPLE_RATE)
        );
        self.event_tx
            .send(MixerEvent::Source(Box::new(source)))
            .unwrap();
        Ok(())
    }

    pub fn start_recording(&mut self) {
        self.event_tx.send(MixerEvent::Start).unwrap();
        self.is_recording = true;
    }

    pub fn stop_recording_and_playback(&mut self) -> Vec<f32> {
        self.event_tx.send(MixerEvent::Stop).unwrap();
        self.is_recording = false;
        self.recording_rx.try_iter().collect()
    }
}

pub struct Mixer {
    event_rx: Receiver<MixerEvent>,
    sources: Vec<BoxedSource>,
    is_recording: bool,
    recording_tx: Sender<f32>,
}

impl Mixer {
    fn new(event_rx: Receiver<MixerEvent>, recording_tx: Sender<f32>) -> Self {
        Mixer {
            event_rx,
            sources: vec![],
            is_recording: false,
            recording_tx,
        }
    }
}

impl Iterator for Mixer {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.event_rx.try_iter().for_each(|e| match e {
            MixerEvent::Source(s) => {
                self.sources.push(s);
            }
            MixerEvent::Start => {
                self.is_recording = true;
            }
            MixerEvent::Stop => {
                self.is_recording = false;
                self.sources.clear();
            }
        });

        // noise appears to be coming from here. why?
        let sample = self
            .sources
            .iter_mut()
            .map(|s| s.next().unwrap_or_default())
            .sum();

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
