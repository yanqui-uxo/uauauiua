use std::collections::HashMap;

use uiua::{value_to_wav_bytes, NativeSys, SysBackend, Uiua};

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioData {
    stack_wav: Vec<u8>,
    var_wavs: HashMap<String, Vec<u8>>,
}

#[derive(Clone)]
pub struct UiuaWrapper(Uiua);

impl UiuaWrapper {
    fn new_uiua() -> Uiua {
        Uiua::with_native_sys()
    }

    pub fn new() -> Self {
        UiuaWrapper(UiuaWrapper::new_uiua())
    }

    // resets before running code
    pub fn run_str(&mut self, code: &str) -> Result<AudioData, String> {
        self.0 = UiuaWrapper::new_uiua();
        self.0.run_str(code).map_err(|e| e.message())?;

        let stack_wav = value_to_wav_bytes(
            &self.0.pop("audio value").map_err(|e| e.message())?,
            NativeSys.audio_sample_rate(),
        )?;
        let var_wavs = self
            .0
            .bound_values()
            .into_iter()
            .filter_map(|(k, v)| {
                Some((
                    k.to_string(),
                    value_to_wav_bytes(&v, NativeSys.audio_sample_rate()).ok()?,
                ))
            })
            .collect();

        Ok(AudioData {
            stack_wav,
            var_wavs,
        })
    }
}
