use std::{println, thread, time::Duration};
use sysinfo::{System, get_current_pid};

mod runners;
mod config_manager;
mod metrics_utilities;


fn main() {
    let config = config_manager::load_config().expect("Failed to load configuration");

    let mut sys = System::new_all();
    let pid = get_current_pid().expect("Failed to get current process PID");
    
    println!("[Parent] Parent process started. Spawning child process every {} seconds...", config.interval_seconds);
    
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