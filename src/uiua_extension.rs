use crate::limited_backend::LimitedBackend;
use crate::recording::{CHANNEL_NUM, SAMPLE_RATE};

use anyhow::{anyhow, ensure};
use crossterm::event::KeyCode;
use regex::Regex;
use rodio::buffer::SamplesBuffer;
use std::{collections::HashMap, sync::LazyLock};
use uiua::Uiua;

pub const MAIN_PATH: &str = "main.ua";

static KEY_FUNCTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^On([A-Z])Press$").unwrap());

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

// TODO: handle errors from value_to_source
fn get_key_sources(uiua: &Uiua) -> HashMap<KeyCode, SamplesBuffer<f32>> {
    uiua.bound_values()
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
        self.key_sources = get_key_sources(&uiua);
        Ok(())
    }

    pub fn key_sources(&self) -> &HashMap<KeyCode, SamplesBuffer<f32>> {
        &self.key_sources
    }
}
