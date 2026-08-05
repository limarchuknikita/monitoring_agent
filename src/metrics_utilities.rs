use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

/// Returns the RSS (Resident Set Size) in MB for a given PID.
pub fn rss_mb_for_pid(sys: &mut System, pid: sysinfo::Pid) -> Option<f64> {
    let refresh_kind = ProcessRefreshKind::nothing().with_memory();

    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        refresh_kind,
    );

    sys.process(pid).map(|proc_| {
        let rss_bytes = proc_.memory();
        rss_bytes as f64 / 1024.0 / 1024.0 // Convert to MB
    })
}