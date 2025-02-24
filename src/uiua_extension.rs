use crate::limited_backend::LimitedBackend;
use crate::recording::{CHANNEL_NUM, SAMPLE_RATE};

use anyhow::{anyhow, bail, ensure};
use crossterm::event::KeyCode;
use rodio::buffer::SamplesBuffer;
use std::collections::HashMap;
use std::time::Duration;
use uiua::{Uiua, Value};

pub const MAIN_PATH: &str = "main.ua";

fn value_to_source(value: &Value) -> anyhow::Result<SamplesBuffer<f32>> {
    let mut value = value.clone();
    while let Value::Box(_) = value {
        value.unbox();
    }

    let mut array = match value {
        Value::Byte(x) => Ok(x.convert::<f64>()),
        Value::Num(x) => Ok(x),
        Value::Box(_) => panic!("array should have already been unboxed"),
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

const KEY_MAP_NAME: &str = "OnPress";
fn get_key_sources(uiua: &Uiua) -> anyhow::Result<HashMap<KeyCode, SamplesBuffer<f32>>> {
    let vals = uiua.bound_values();
    let map = vals
        .get(KEY_MAP_NAME)
        .ok_or(anyhow!("could not get value {KEY_MAP_NAME}"))?;

    ensure!(map.is_map(), "{KEY_MAP_NAME} is not a map");

    map.map_kv()
        .into_iter()
        .map(|(k, v)| {
            // TODO: make pull request so that requirements are an Option
            let name = k.as_string(uiua, "")?;
            if name.chars().count() == 1 {
                let c = name.chars().next().unwrap();
                if c.is_ascii_uppercase() {
                    bail!("expected '{c}' to be lowercase");
                }
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

const EXECUTION_TIME_LIMIT: u64 = 10;
impl UiuaExtension {
    pub fn load(&mut self) -> anyhow::Result<()> {
        let mut uiua = Uiua::with_backend(LimitedBackend)
            .with_execution_limit(Duration::from_secs(EXECUTION_TIME_LIMIT));
        uiua.run_file(MAIN_PATH)?;
        self.key_sources = get_key_sources(&uiua)?;
        Ok(())
    }

    pub fn key_sources(&self) -> &HashMap<KeyCode, SamplesBuffer<f32>> {
        &self.key_sources
    }
}
