use crate::{config_manager, metrics_utilities};
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use sysinfo::{get_current_pid, System};
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::{define_windows_service, service_dispatcher};

const SERVICE_NAME: &str = "monitoring_agent";

define_windows_service!(ffi_service_main, service_main);

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

pub fn install_service() -> io::Result<()> {
    let current_exe = std::env::current_exe()?;
    let bin_path = current_exe.to_string_lossy().to_string();

    let delete_status = Command::new("sc")
        .args(["delete", SERVICE_NAME])
        .status();

    if let Ok(status) = delete_status {
        if status.success() {
            println!("[Install] Removed existing service 'monitoring_agent'.");
        }
    }

    let create_status = Command::new("sc")
        .args([
            "create",
            SERVICE_NAME,
            "binPath=",
            &bin_path,
            "start=",
            "auto",
            "DisplayName=",
            "Monitoring Agent",
        ])
        .status()?;

    if !create_status.success() {
        return Err(io::Error::other("'sc create' failed"));
    }

    println!("[Install] Service '{}' installed successfully.", SERVICE_NAME);
    println!("[Install] Start it with: sc.exe start {}", SERVICE_NAME);

    Ok(())
}

pub fn try_run_service_mode() -> io::Result<bool> {
    match service_dispatcher::start(SERVICE_NAME, ffi_service_main) {
        Ok(()) => Ok(true),
        Err(err) => {
            let text = err.to_string();
            if text.contains("1063") || text.contains("ERROR_FAILED_SERVICE_CONTROLLER_CONNECT") {
                Ok(false)
            } else {
                Err(io::Error::other(format!(
                    "failed to start service dispatcher: {err}"
                )))
            }
        }
    }
}

fn service_main(_args: Vec<OsString>) {
    if let Err(err) = run_service_loop() {
        eprintln!("[Service] Fatal error: {}", err);
    }
}

fn run_service_loop() -> io::Result<()> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

    let status_handle = service_control_handler::register(SERVICE_NAME, move |control_event| {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                let _ = shutdown_tx.send(());
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    })
    .map_err(|err| io::Error::other(format!("service control registration failed: {err}")))?;

    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::StartPending,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 1,
            wait_hint: Duration::from_secs(10),
            process_id: None,
        })
        .map_err(|err| io::Error::other(format!("failed to set start pending: {err}")))?;

    let config = config_manager::load_config()
        .map_err(|err| io::Error::other(format!("Failed to load configuration: {err}")))?;

    prepare_log_file(&config.log_file_path)?;

    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .map_err(|err| io::Error::other(format!("failed to set running state: {err}")))?;

    let mut sys = System::new_all();
    let pid = get_current_pid().map_err(|err| io::Error::other(format!("pid error: {err}")))?;

    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        let time_utc = chrono::Utc::now();
        let rss_metric = metrics_utilities::rss_mb_for_pid(&mut sys, pid).unwrap_or(0.0);
        let metrics = format!("{} | {:.2} MB", time_utc.format("%Y-%m-%d %H:%M:%S"), rss_metric);

        if let Err(err) = spawn_child_process(&config.child_binary_name, metrics) {
            eprintln!("[Service] Failed to spawn child process: {}", err);
        }

        if shutdown_rx
            .recv_timeout(Duration::from_secs(config.interval_seconds))
            .is_ok()
        {
            break;
        }
    }

    status_handle
        .set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .map_err(|err| io::Error::other(format!("failed to set stopped state: {err}")))?;

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