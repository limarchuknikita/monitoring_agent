use std::process::{Command, Child};
use std::io;

pub fn spawn_child_process(child_binary_name: &str, metrics: &[String]) -> io::Result<Child> {
    Command::new("powershell")
        .args(&["-Command", "Start-Process", &format!("\"{}.exe\"", child_binary_name), "-Verb", "RunAs"])
        .args(metrics)
        .spawn()
}