use anyhow::Result;
use chrono::Utc;
use std::{collections::HashMap, env, fs};

pub async fn run() -> Result<()> {
    let root = env::current_dir()?;
    let log_file = root.join("devflow.log");
    if !log_file.exists() {
        println!("No devflow.log found");
        return Ok(());
    }

    let content = fs::read_to_string(&log_file)?;
    let mut groups: HashMap<String, usize> = HashMap::new();
    for line in content.lines() {
        if line.contains("ERROR") || line.contains("panic") {
            *groups.entry(normalize_trace(line)).or_insert(0) += 1;
        }
    }

    println!("Grouped errors:");
    for (trace, count) in &groups {
        println!("freq={} trace={}", count, trace);
    }

    let state_path = root.join(".devflow/last_logs_state.json");
    fs::create_dir_all(root.join(".devflow"))?;
    if state_path.exists() {
        let old = fs::read_to_string(&state_path)?;
        let old_groups: HashMap<String, usize> = serde_json::from_str(&old)?;
        for k in groups.keys() {
            if !old_groups.contains_key(k) {
                println!("new_error_since_last_run: {}", k);
            }
        }
    }

    fs::write(state_path, serde_json::to_string_pretty(&groups)?)?;
    println!("first_seen_reference: {}", Utc::now());
    Ok(())
}

fn normalize_trace(line: &str) -> String {
    line.split_whitespace()
        .map(|t| if t.chars().all(|c| c.is_numeric()) { "<n>" } else { t })
        .collect::<Vec<_>>()
        .join(" ")
}
