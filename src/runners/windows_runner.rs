use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;
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

    if let Err(err) = restrict_log_acl_windows(log_file) {
        eprintln!("[Warning] Failed to restrict log file ACL: {}", err);
    }

    Ok(())
}

fn restrict_log_acl_windows(log_file: &str) -> io::Result<()> {
    let status1 = Command::new("icacls")
        .args([log_file, "/inheritance:r"])
        .status()?;

    let status2 = Command::new("icacls")
        .args([
            log_file,
            "/grant:r",
            "*S-1-5-18:(F)",
            "*S-1-5-32-544:(F)",
        ])
        .status()?;

    let status3 = Command::new("icacls")
        .args([
            log_file,
            "/remove:g",
            "*S-1-1-0",
            "*S-1-5-11",
            "Users",
            "Everyone",
            "Authenticated Users",
        ])
        .status()?;

    if status1.success() && status2.success() && status3.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to apply ACL with icacls",
        ))
    }
}