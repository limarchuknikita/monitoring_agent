use std::io;
use std::process::{Child, Command, Stdio};

pub fn spawn_child_process(
    child_binary_path: &str,
    metrics: String,
) -> io::Result<Child> {
    let mut cmd = Command::new(child_binary_path);
    cmd.arg(metrics);

    // On Windows, the child inherits the parent's security token.
    // If the parent runs elevated, the child runs elevated too.
    // Stdout/stderr are inherited so logs appear in the parent console.
    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}