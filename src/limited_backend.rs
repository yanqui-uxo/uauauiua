use std::{
    any::Any,
    path::{Path, PathBuf},
};

use uiua::{GitTarget, Handle, NativeSys, ReadLinesReturnFn, SysBackend};

macro_rules! native_call_methods {
    ($($name:ident($($arg:ident: $arg_type:ty),*) -> $ret_type:ty;)+) => {
		$(fn $name(&self, $($arg: $arg_type),*) -> $ret_type {
			NativeSys.$name($($arg),*)
		})+
	};
}

pub struct LimitedBackend;
impl SysBackend for LimitedBackend {
    fn any(&self) -> &dyn Any {
        self
    }
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn read_lines<'a>(&self, handle: Handle) -> Result<ReadLinesReturnFn<'a>, String> {
        NativeSys.read_lines(handle)
    }

    native_call_methods!(
        change_directory(path: &str) -> Result<(), String>;
        clipboard() -> Result<String, String>;
        create_file(path: &Path) -> Result<Handle, String>;
        delete(path: &str) -> Result<(), String>;
        file_exists(path: &str) -> bool;
        file_read_all(path: &Path) -> Result<Vec<u8>, String>;
        file_write_all(path: &Path, contents: &[u8]) -> Result<(), String>;
        is_file(path: &str) -> Result<bool, String>;
        list_dir(path: &str) -> Result<Vec<String>, String>;
        load_git_module(url: &str, target: GitTarget) -> Result<PathBuf, String>;
        make_dir(path: &Path) -> Result<(), String>;
        open_file(path: &Path, write: bool) -> Result<Handle, String>;
        read(handle: Handle, count: usize) -> Result<Vec<u8>, String>;
        read_all(handle: Handle) -> Result<Vec<u8>, String>;
        read_until(handle: Handle, delim: &[u8]) -> Result<Vec<u8>, String>;
        set_clipboard(contents: &str) -> Result<(), String>;
        write(handle: Handle, contents: &[u8]) -> Result<(), String>;
    );
}
