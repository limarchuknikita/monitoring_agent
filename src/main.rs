use std::{env, println, thread, time::Duration};
use sysinfo::{get_current_pid, System};

mod config_manager;
mod metrics_utilities;
mod runners;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--install") {
        if let Err(err) = runners::install_service() {
            eprintln!("[Error] Failed to install service: {}", err);
            std::process::exit(1);
        }
        return;
    }

    match runners::try_run_service_mode() {
        Ok(true) => return,
        Ok(false) => {}
        Err(err) => {
            eprintln!("[Error] Failed to start service mode: {}", err);
            std::process::exit(1);
        }
    }

    let config = config_manager::load_config().expect("Failed to load configuration");
    runners::prepare_log_file(&config.log_file_path).expect("Failed to prepare log file path");

    let mut sys = System::new_all();
    let pid = get_current_pid().expect("Failed to get current process PID");

    println!(
        "[Parent] Parent process started. Spawning child process every {} seconds...",
        config.interval_seconds
    );

    loop {
        let time_utc = chrono::Utc::now();
        let rss_metric = metrics_utilities::rss_mb_for_pid(&mut sys, pid).unwrap_or(0.0);
        let metrics = format!("{} | {:.2} MB", time_utc.format("%Y-%m-%d %H:%M:%S"), rss_metric);

        if let Err(e) = runners::spawn_child_process(&config.child_binary_name, metrics) {
            eprintln!("[Error] Failed to spawn child process: {}", e);
        }

        thread::sleep(Duration::from_secs(config.interval_seconds));
    }
}