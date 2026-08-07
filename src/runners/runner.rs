#[cfg(target_os = "windows")]
#[path = "windows_runner.rs"]
mod runners;

#[cfg(target_os = "linux")]
#[path = "linux_runner.rs"]
mod runners;

#[cfg(target_os = "macos")]
#[path = "macos_runner.rs"]
mod runners;

use std::process::Child;
use std::io;

pub fn spawn_child_process(child_binary_name: &str, metrics: String) -> io::Result<Child> {
    runners::spawn_child_process(child_binary_name, metrics)
}