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

    pub fn reset(&mut self) {
        self.0 = UiuaWrapper::new_uiua();
    }

    pub fn pop(&mut self, arg: impl uiua::StackArg) -> Result<uiua::Value, String> {
        self.0.pop(arg).map_err(|e| e.message())
    }

    pub fn run_str(&mut self, code: &str) -> Result<(), String> {
        self.0.run_str(code).map_err(|e| e.message())?;
        Ok(())
    }
}
