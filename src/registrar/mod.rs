use std::path::PathBuf;

use crate::error::Result;

pub struct ServiceSpec {
    pub label: String,
    /// rebooted バイナリの絶対パス
    pub program: PathBuf,
    /// `--internal-exec <label> -- <cmd> [args...]` 形式の引数列
    pub args: Vec<String>,
}

pub trait Registrar {
    fn register(&self, spec: &ServiceSpec) -> Result<()>;
    fn unregister(&self, label: &str) -> Result<()>;
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub fn new_registrar() -> Box<dyn Registrar> {
    Box::new(macos::LaunchAgentRegistrar)
}

#[cfg(target_os = "linux")]
pub fn new_registrar() -> Box<dyn Registrar> {
    linux::new_registrar()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn new_registrar() -> Box<dyn Registrar> {
    compile_error!("サポートされていないOS: macOS または Linux が必要です")
}
