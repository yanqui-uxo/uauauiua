use crate::limited_backend::LimitedBackend;
use crate::recording::{CHANNEL_NUM, SAMPLE_RATE};

use anyhow::{anyhow, ensure};
use crossterm::event::KeyCode;
use rodio::buffer::SamplesBuffer;
use std::collections::HashMap;
use uiua::Uiua;

pub const MAIN_PATH: &str = "main.ua";

fn value_to_source(value: &uiua::Value) -> anyhow::Result<SamplesBuffer<f32>> {
    let mut array = match value {
        uiua::Value::Byte(x) => Ok(x.convert_ref::<f64>()),
        uiua::Value::Num(x) => Ok(x.clone()),
        _ => Err(anyhow!("audio array must be non-complex numeric")),
    }?;

    ensure!(
        array.rank() == 2 && array.shape().dims()[0] == CHANNEL_NUM as usize,
        "audio array shape must be of form [{} n]",
        CHANNEL_NUM
    );

    array.transpose();

    #[allow(clippy::cast_possible_truncation)]
    let array = array.convert_with(|x| x as f32);

    let array_vec: Vec<_> = array.elements().copied().collect();

    Ok(SamplesBuffer::new(CHANNEL_NUM, *SAMPLE_RATE, array_vec))
}

fn get_key_sources(uiua: &Uiua) -> anyhow::Result<HashMap<KeyCode, SamplesBuffer<f32>>> {
    const KEY_MAP_NAME: &str = "OnPress";

    let vals = uiua.bound_values();
    let map = vals
        .get(KEY_MAP_NAME)
        .ok_or(anyhow!("could not get value {KEY_MAP_NAME}"))?;

    ensure!(map.is_map(), "{KEY_MAP_NAME} is not a map");

    map.map_kv()
        .into_iter()
        .map(|(k, v)| {
            let name = k.as_string(uiua, "")?;
            if name.chars().count() == 1 {
                Ok((
                    KeyCode::Char(name.chars().next().unwrap()),
                    value_to_source(&v)?,
                ))
            } else {
                Err(anyhow!("expected '{k}' to be one character"))
            }
        })
        .collect()
}

#[derive(Default)]
pub struct UiuaExtension {
    key_sources: HashMap<KeyCode, SamplesBuffer<f32>>,
}

impl UiuaExtension {
    pub fn load(&mut self) -> anyhow::Result<()> {
        let mut uiua = Uiua::with_backend(LimitedBackend);
        uiua.run_file(MAIN_PATH)?;
        self.key_sources = get_key_sources(&uiua)?;
        Ok(())
    }

    pub fn key_sources(&self) -> &HashMap<KeyCode, SamplesBuffer<f32>> {
        &self.key_sources
    }
}
