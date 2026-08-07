#[cfg(target_os = "windows")]
#[path = "windows_runner.rs"]
mod runners;

#[cfg(target_os = "linux")]
#[path = "linux_runner.rs"]
mod runners;

#[cfg(target_os = "macos")]
#[path = "macos_runner.rs"]
mod runners;

use std::io;
use std::process::Child;

pub fn spawn_child_process(child_binary_name: &str, metrics: String) -> io::Result<Child> {
    runners::spawn_child_process(child_binary_name, metrics)
}

pub fn prepare_log_file(log_file: &str) -> io::Result<()> {
    runners::prepare_log_file(log_file)
}

pub fn install_service() -> io::Result<()> {
    runners::install_service()
}

pub fn try_run_service_mode() -> io::Result<bool> {
    runners::try_run_service_mode()
}