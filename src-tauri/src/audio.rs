use std::sync::{Arc, Mutex};

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
}

impl Player {
    pub fn new(handle: &OutputStreamHandle) -> Self {
        let sink = Sink::try_new(handle).unwrap();
        sink.pause();
        Player {
            state: PlayerState::Stopped,
            sink,
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

    pub fn load_value(&mut self, value: &uiua::Value) -> Result<(), String> {
        self.sink.clear();
        let data: Vec<f32> = uiua::value_to_sample(value)?
            .into_iter()
            .flatten()
            .collect();
        self.sink
            .append(SamplesBuffer::new(2, NativeSys.audio_sample_rate(), data));
        Ok(())
    }
}

#[taurpc::procedures(path = "audio")]
pub trait AudioApi {
    async fn play_from_stack() -> Result<(), String>;
}

#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct AudioApiImpl {
    pub player: Arc<Mutex<Player>>,
    pub uiua: Arc<Mutex<UiuaWrapper>>,
}

#[taurpc::resolvers(export_to = "../src/types.ts")]
impl AudioApi for AudioApiImpl {
    async fn play_from_stack(self) -> Result<(), String> {
        let mut uiua = self.uiua.lock().unwrap();
        let mut player = self.player.lock().unwrap();
        player.load_value(&uiua.pop("audio value")?)?;
        player.handle_action(PlayerAction::Play);
        Ok(())
    }
}
