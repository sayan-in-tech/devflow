use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use sysinfo::{ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcSnapshot {
    pub pid: u32,
    pub name: String,
    pub cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub saved_at: DateTime<Utc>,
    pub cwd: String,
    pub processes: Vec<ProcSnapshot>,
    pub env: Vec<(String, String)>,
}

pub fn save_snapshot(root: &Path) -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let cwd = root.display().to_string();

    let processes = sys
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            let cmd = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            if cmd.contains(&cwd) || process.name().to_string_lossy().contains("cargo") {
                Some(ProcSnapshot {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    cmd,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let env = std::env::vars()
        .filter(|(k, _)| {
            !k.to_lowercase().contains("token") && !k.to_lowercase().contains("secret")
        })
        .collect::<Vec<_>>();

    let snap = Snapshot {
        saved_at: Utc::now(),
        cwd,
        processes,
        env,
    };

    let content = serde_json::to_string_pretty(&snap)?;
    fs::create_dir_all(root.join(".devflow"))?;
    fs::write(root.join(".devflow/snapshot.json"), content)?;
    Ok(())
}

pub fn read_snapshot(root: &Path) -> Result<Snapshot> {
    let content = fs::read_to_string(root.join(".devflow/snapshot.json"))?;
    Ok(serde_json::from_str(&content)?)
}
