use std::collections::HashMap;

use uiua::Uiua;

#[derive(Clone)]
pub struct UiuaWrapper(Uiua);

impl UiuaWrapper {
    fn new_uiua() -> Uiua {
        Uiua::with_native_sys()
    }

    pub fn new() -> Self {
        UiuaWrapper(UiuaWrapper::new_uiua())
    }

    pub fn pop(&mut self, arg: impl uiua::StackArg) -> Result<uiua::Value, String> {
        self.0.pop(arg).map_err(|e| e.message())
    }

    // resets before running code
    pub fn run_str(&mut self, code: &str) -> Result<(), String> {
        self.0 = UiuaWrapper::new_uiua();
        self.0.run_str(code).map_err(|e| e.message())?;
        Ok(())
    }

    pub fn value_to_sample(value: &uiua::Value) -> Result<Vec<f32>, String> {
        uiua::value_to_sample(value).map(|v| v.into_iter().flatten().collect())
    }

    pub fn bound_samples(&self) -> HashMap<String, Vec<f32>> {
        self.0
            .bound_values()
            .into_iter()
            .filter_map(|(k, v)| Some((k.to_string(), UiuaWrapper::value_to_sample(&v).ok()?)))
            .collect()
    }
}
