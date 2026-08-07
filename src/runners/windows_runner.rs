use std::io;
use std::process::{Child, Command, Stdio};
use std::thread;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub fn spawn_child_process(
    child_binary_path: &str,
    metrics: &[String],
) -> io::Result<Child> {
    let mut cmd = Command::new(child_binary_path);
    cmd.args(metrics);

    cmd.stdout(Stdio::piped())
       .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        const DETACHED_PROCESS: u32 = 0x00000008;
        cmd.creation_flags(DETACHED_PROCESS);
    }

    let mut child = cmd.spawn()?;

    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    println!("[Go Stdout]: {}", l);
                }
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(l) = line {
                    eprintln!("[Go Stderr]: {}", l);
                }
            }
        });
    }

    Ok(child)
}