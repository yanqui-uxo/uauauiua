use crate::recording::{Mixer, MixerController};
use crate::uiua_extension::UiuaExtension;

use rodio::OutputStream;

struct Uauauiua {
    uiua_extension: UiuaExtension,
    _stream: OutputStream,
    mixer: Mixer,
    mixer_controller: MixerController,
}

impl Uauauiua {
    pub fn new() -> Self {
        todo!();
    }
}
