use anyhow::Result;
use std::env;

use crate::utils::snapshot::{read_snapshot, save_snapshot};

pub async fn save() -> Result<()> {
    let root = env::current_dir()?;
    save_snapshot(&root)?;
    println!("snapshot saved to .devflow/snapshot.json");
    Ok(())
}

pub async fn restore() -> Result<()> {
    let root = env::current_dir()?;
    let snap = read_snapshot(&root)?;
    println!("snapshot from {}", snap.saved_at);
    println!("repo: {}", snap.cwd);
    for p in snap.processes {
        println!("would restore: {} {}", p.name, p.cmd);
    }
    Ok(())
}
