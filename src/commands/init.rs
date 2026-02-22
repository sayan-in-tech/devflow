use anyhow::Result;
use std::env;

use crate::utils::config::write_default_config;

pub async fn run() -> Result<()> {
    let root = env::current_dir()?;
    if root.join(".devflow.yaml").exists() {
        println!(".devflow.yaml already exists");
        return Ok(());
    }
    write_default_config(&root)?;
    println!("Created .devflow.yaml");
    Ok(())
}
