use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;
use std::process::{Command, Child};

pub fn spawn_child_process(child_binary_name: &str, metrics: String) -> std::io::Result<Child> {
    Command::new(child_binary_name)
        .arg(metrics)
        .spawn()
}

pub fn prepare_log_file(log_file: &str) -> io::Result<()> {
    let path = Path::new(log_file);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    Ok(())
}

pub fn install_service() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "--install is only supported on Windows",
    ))
}

pub fn try_run_service_mode() -> io::Result<bool> {
    Ok(false)
}