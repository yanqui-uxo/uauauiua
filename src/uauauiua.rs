use anyhow::{anyhow, ensure};
use crossterm::event::KeyCode;
use regex::Regex;
use rodio::buffer::SamplesBuffer;
use std::collections::HashMap;
use std::sync::LazyLock;
use uiua::{NativeSys, SysBackend, Uiua};

pub const MAIN_PATH: &str = "main.ua";

pub struct Uauauiua {
    uiua: Uiua,
    key_sources: HashMap<KeyCode, SamplesBuffer<f32>>,
}

static KEY_FUNCTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^On([A-Z])Press$").unwrap());

fn value_to_source(value: &uiua::Value) -> anyhow::Result<SamplesBuffer<f32>> {
    let mut array = match value {
        uiua::Value::Byte(x) => Ok(x.convert_ref::<f64>()),
        uiua::Value::Num(x) => Ok(x.clone()),
        _ => Err(anyhow!("audio array must be non-complex numeric")),
    }?;

    ensure!(
        matches!(array.rank(), 1 | 2),
        "audio array rank must be 1 or 2"
    );

    if array.rank() == 1 {
        array.fix();
    }

    let channels_num: u16 = array.shape().dims()[0]
        .try_into()
        .map_err(|_| anyhow!("far too many channels. what the heck are you doing?"))?;

    array.transpose();

    #[allow(clippy::cast_possible_truncation)]
    let array = array.convert_with(|x| x as f32);

    let array_vec: Vec<_> = array.elements().copied().collect();

    Ok(SamplesBuffer::new(
        channels_num,
        NativeSys.audio_sample_rate(),
        array_vec,
    ))
}

impl Uauauiua {
    pub fn new() -> anyhow::Result<Uauauiua> {
        let mut uiua = Uiua::with_safe_sys();
        uiua.run_file(MAIN_PATH)?;

        // TODO: don't fail silently
        let key_sources: HashMap<KeyCode, SamplesBuffer<f32>> = uiua
            .bound_values()
            .into_iter()
            .filter_map(|(name, v)| {
                Some((
                    KeyCode::Char(
                        KEY_FUNCTION_REGEX
                            .captures(&name)?
                            .get(1)?
                            .as_str()
                            .chars()
                            .next()
                            .unwrap()
                            .to_ascii_lowercase(),
                    ),
                    value_to_source(&v).ok()?,
                ))
            })
            .collect();

        Ok(Uauauiua { uiua, key_sources })
    }

    pub fn uiua(&self) -> &Uiua {
        &self.uiua
    }
    pub fn key_sources(&self) -> &HashMap<KeyCode, SamplesBuffer<f32>> {
        &self.key_sources
    }
}
