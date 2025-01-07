use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, LazyLock,
    },
    time::Duration,
};

use anyhow::ensure;
use rodio::Source;
use uiua::{NativeSys, SysBackend};

type BoxedSource = Box<dyn Source<Item = f32> + Send>;

pub fn new_mixer() -> (Arc<MixerController>, Mixer) {
    let (tx, rx) = channel();
    (Arc::new(MixerController::new(tx)), Mixer::new(rx))
}

const CHANNEL_NUM: u16 = 2;
static SAMPLE_RATE: LazyLock<u32> = LazyLock::new(|| NativeSys.audio_sample_rate());

pub struct MixerController {
    tx: Sender<BoxedSource>,
}
impl MixerController {
    fn new(tx: Sender<BoxedSource>) -> Self {
        MixerController { tx }
    }

    pub fn add<T>(&self, source: T) -> anyhow::Result<()>
    where
        T: Source<Item = f32> + Send + 'static,
    {
        ensure!(
            source.channels() == CHANNEL_NUM,
            "incorrect number of channels"
        );
        ensure!(
            source.sample_rate() == *SAMPLE_RATE,
            "incorrect sample rate"
        );
        self.tx.send(Box::new(source)).unwrap();
        Ok(())
    }
}

pub struct Mixer {
    rx: Receiver<BoxedSource>,
    sources: Vec<BoxedSource>,
}

impl Iterator for Mixer {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        while let Ok(s) = self.rx.try_recv() {
            self.sources.push(s);
        }

        Some(
            self.sources
                .iter_mut()
                .fold(0., |acc, s| acc + s.next().unwrap_or(0.)),
        )
    }
}

impl Source for Mixer {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        NativeSys.audio_sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Mixer {
    fn new(rx: Receiver<BoxedSource>) -> Self {
        Mixer {
            rx,
            sources: vec![],
        }
    }
}
