use std::collections::HashMap;

use uiua::{value_to_audio_channels, value_to_wav_bytes, NativeSys, SysBackend, Uiua};

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct Clip {
    wav: Vec<u8>,
    peaks: Vec<Vec<f64>>,
}

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioData {
    stack_clips: Vec<Clip>,
    var_clips: HashMap<String, Clip>,
}

fn value_to_clip(value: &uiua::Value) -> Result<Clip, String> {
    let wav = value_to_wav_bytes(value, NativeSys.audio_sample_rate())?;
    let peaks = value_to_audio_channels(value)?;
    Ok(Clip { wav, peaks })
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

        let stack_clips = self
            .0
            .pop("audio values")
            .map_err(|e| e.message())?
            .rows()
            .map(|v| value_to_clip(&v))
            .collect::<Result<Vec<_>, String>>()?;
        let var_clips = self
            .0
            .bound_values()
            .into_iter()
            .filter_map(|(k, v)| Some((k.to_string(), value_to_clip(&v).ok()?)))
            .collect();

        Ok(AudioData {
            stack_clips,
            var_clips,
        })
    }
}
