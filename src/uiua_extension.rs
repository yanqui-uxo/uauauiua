use crate::recording::{CHANNEL_NUM, SAMPLE_RATE};

use anyhow::{anyhow, ensure};
use crossterm::event::KeyCode;
use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use rodio::buffer::SamplesBuffer;
use std::{collections::HashMap, path::Path, sync::LazyLock};
use tokio::sync::watch::{self, Ref};
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

pub struct UiuaExtensionData {
    key_sources: HashMap<KeyCode, SamplesBuffer<f32>>,
}

impl UiuaExtensionData {
    pub fn key_sources(&self) -> &HashMap<KeyCode, SamplesBuffer<f32>> {
        &self.key_sources
    }
}

impl From<Uiua> for UiuaExtensionData {
    fn from(value: Uiua) -> Self {
        let key_sources = value
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
        UiuaExtensionData { key_sources }
    }
}

pub struct UiuaExtension {
    data_rx: watch::Receiver<anyhow::Result<UiuaExtensionData>>,
    watcher: RecommendedWatcher,
    pub new_values: HashMap<String, uiua::Value>,
}

impl UiuaExtension {
    pub fn new() -> Self {
        let (data_tx, data_rx) = watch::channel(Self::load());
        let mut watcher = recommended_watcher(move |e: Result<notify::Event, _>| {
            if let notify::EventKind::Modify(_) = e.unwrap().kind {
                data_tx.send(Self::load()).expect("should have sent data");
            }
        })
        .expect("should have initialized file watcher");
        watcher.watch(Path::new(MAIN_PATH), RecursiveMode::NonRecursive);

        UiuaExtension {
            data_rx,
            watcher,
            new_values: HashMap::new(),
        }
    }

    fn load() -> anyhow::Result<UiuaExtensionData> {
        let mut uiua = Uiua::with_safe_sys();
        uiua.run_file(MAIN_PATH)?;
        Ok(uiua.into())
    }

    fn data(&self) -> Ref<anyhow::Result<UiuaExtensionData>> {
        self.data_rx.borrow()
    }
}
