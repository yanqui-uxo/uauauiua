use crate::limited_backend::LimitedBackend;
use crate::recording::{CHANNEL_NUM, SAMPLE_RATE};

use anyhow::{anyhow, bail, ensure};
use crossterm::event::KeyCode;
use indexmap::IndexMap;
use rodio::buffer::SamplesBuffer;
use std::time::Duration;
use uiua::{Uiua, Value};

pub const MAIN_PATH: &str = "main.ua";
const KEY_MAP_NAME: &str = "OnPress";
const EXECUTION_TIME_LIMIT: u64 = 5;

fn value_to_source(value: &Value, key: char) -> anyhow::Result<SamplesBuffer<f32>> {
    let value = value.clone().unpacked();

    let array = match value {
        Value::Byte(x) => x.convert::<f64>(),
        Value::Num(x) => x,
        _ => bail!("value for key '{key}' of {KEY_MAP_NAME} must be non-complex numeric"),
    };

    let shape = array.shape();
    ensure!(
        array.rank() == 2 && shape.dims()[1] == CHANNEL_NUM as usize,
        "shape {shape} for key '{key}' of {KEY_MAP_NAME} is not of form [n {CHANNEL_NUM}]",
    );

    #[allow(clippy::cast_possible_truncation)]
    let array = array.convert_with(|x| x as f32);

    let array_vec: Vec<_> = array.elements().copied().collect();

    Ok(SamplesBuffer::new(CHANNEL_NUM, *SAMPLE_RATE, array_vec))
}

fn get_key_sources(uiua: &mut Uiua) -> anyhow::Result<IndexMap<KeyCode, SamplesBuffer<f32>>> {
    let vals = uiua.bound_values();
    let funcs = uiua.bound_functions();

    let owned_map;
    let map;
    if let Some(m) = vals.get(KEY_MAP_NAME) {
        map = m;
    } else if let Some(f) = funcs.get(KEY_MAP_NAME) {
        uiua.call(f)?;
        owned_map = uiua.pop("keyboard map")?;
        map = &owned_map;
    } else {
        bail!("Could not get OnPress");
    }

    ensure!(map.is_map(), "{KEY_MAP_NAME} is not a map");

    map.map_kv()
        .into_iter()
        .map(|(k, v)| {
            let name = k.as_string(uiua, None)?;
            if name.chars().count() == 1 {
                let c = name.chars().next().unwrap();
                ensure!(
                    c.is_ascii_lowercase(),
                    "expected '{c}' in {KEY_MAP_NAME} keys to be lowercase ASCII"
                );
                Ok((KeyCode::Char(c), value_to_source(&v, c)?))
            } else {
                Err(anyhow!(
                    "expected {KEY_MAP_NAME} key '{k}' to be one character"
                ))
            }
        })
        .collect()
}

pub struct UiuaExtension {
    uiua: Uiua,
    key_sources: IndexMap<KeyCode, SamplesBuffer<f32>>,
}

impl Default for UiuaExtension {
    fn default() -> Self {
        Self {
            uiua: Uiua::with_backend(LimitedBackend)
                .with_execution_limit(Duration::from_secs(EXECUTION_TIME_LIMIT)),
            key_sources: IndexMap::default(),
        }
    }
}

impl UiuaExtension {
    pub fn load(&mut self) -> anyhow::Result<()> {
        self.uiua.run_file(MAIN_PATH)?;
        self.key_sources = get_key_sources(&mut self.uiua)?;
        Ok(())
    }

    pub fn key_sources(&self) -> &IndexMap<KeyCode, SamplesBuffer<f32>> {
        &self.key_sources
    }

    pub fn stack(&self) -> &[Value] {
        self.uiua.stack()
    }

    pub fn clear_stack(&mut self) {
        self.uiua.take_stack();
    }
}
