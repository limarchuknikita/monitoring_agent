use std::process::{Command, Child};

pub fn spawn_child_process(child_binary_name: &str, metrics: &[String]) -> std::io::Result<Child> {
    Command::new(child_binary_name)
        .args(metrics)
        .spawn()
}