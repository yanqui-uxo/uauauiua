use std::collections::HashMap;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStreamHandle, Sink};
use uiua::{NativeSys, SysBackend};

use crate::uiua_wrapper::UiuaWrapper;

#[derive(Clone, Copy)]
pub enum PlayerState {
    Stopped,
    Playing(u32),
    Paused(u32),
}

pub enum PlayerAction {
    Stop,
    Play,
    Pause,
}

pub struct Player {
    state: PlayerState,
    sink: Sink,
    stack_sample: Vec<f32>,
    bound_samples: HashMap<String, Vec<f32>>,
}

impl Player {
    pub fn new(handle: &OutputStreamHandle) -> Self {
        let sink = Sink::try_new(handle).unwrap();
        sink.pause();
        Player {
            state: PlayerState::Stopped,
            sink,
            stack_sample: Vec::new(),
            bound_samples: HashMap::new(),
        }
    }

    pub fn state(&self) -> PlayerState {
        self.state
    }

    pub fn handle_action(&mut self, action: PlayerAction) {
        match action {
            PlayerAction::Stop => {
                self.sink.pause();
                self.sink.clear();
                self.state = PlayerState::Stopped;
            }
            PlayerAction::Play => self.sink.play(),
            _ => todo!(),
        }
    }

    pub fn load_from_code(&mut self, code: String, uiua: &mut UiuaWrapper) -> Result<(), String> {
        println!("hey ho");
        uiua.run_str(&code)?;
        self.stack_sample = UiuaWrapper::value_to_sample(&uiua.pop("audio value")?)?;
        self.bound_samples = uiua.bound_samples();
        Ok(())
    }

    fn load_sample(&mut self, sample: Vec<f32>) {
        self.sink.clear();
        self.sink
            .append(SamplesBuffer::new(2, NativeSys.audio_sample_rate(), sample));
    }
    pub fn load_stack_sample(&mut self) {
        self.load_sample(self.stack_sample.clone());
    }
    pub fn load_var_sample(&mut self, var: String) {
        self.load_sample(self.bound_samples.get(&var).unwrap().clone());
    }
}
