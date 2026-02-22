use serde::Serialize;
use std::net::TcpListener;
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize)]
pub struct PortOwner {
    pub port: u16,
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub cmd: String,
    pub memory_kb: u64,
    pub uptime_secs: u64,
}

pub fn common_free_ports() -> Vec<u16> {
    [3000, 3001, 5173, 8000, 8080, 5432, 6379]
        .into_iter()
        .filter(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
        .collect()
}

pub fn find_owner_by_port(port: u16) -> Option<PortOwner> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    for (pid, proc_) in sys.processes() {
        if proc_.name().to_string_lossy().contains(&port.to_string())
            || proc_
                .cmd()
                .iter()
                .any(|c| c.to_string_lossy().contains(&port.to_string()))
        {
            let parent_pid = proc_.parent().map(|p| p.as_u32());
            return Some(PortOwner {
                port,
                pid: pid.as_u32(),
                parent_pid,
                cmd: proc_
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" "),
                memory_kb: proc_.memory(),
                uptime_secs: proc_.run_time(),
            });
        }
    }
    None
}

pub fn safe_kill_suggestion(pid: u32) -> Vec<String> {
    vec![
        format!("Try graceful stop first: kill {}", pid),
        format!("If needed force stop: kill -9 {}", pid),
        format!("Windows graceful: taskkill /PID {}", pid),
        format!("Windows force: taskkill /F /PID {}", pid),
    ]
}

pub fn process_name(pid: u32) -> Option<String> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let pid = Pid::from_u32(pid);
    sys.process(pid)
        .map(|p| p.name().to_string_lossy().to_string())
}
