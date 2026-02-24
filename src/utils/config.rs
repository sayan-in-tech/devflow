use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevflowConfig {
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub services: Vec<ServiceDef>,
    #[serde(default)]
    pub start_commands: Vec<String>,
    #[serde(default)]
    pub test_command: Option<String>,
    #[serde(default)]
    pub ignore_globs: Vec<String>,
    #[serde(default)]
    pub desired_ports: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDef {
    pub name: String,
    pub command: String,
}

pub fn load_config(root: &Path) -> Result<DevflowConfig> {
    let path = root.join(".devflow.yaml");
    let content =
        fs::read_to_string(&path).with_context(|| format!("could not read {}", path.display()))?;
    serde_yaml::from_str(&content).context("invalid .devflow.yaml")
}

pub fn write_default_config(root: &Path) -> Result<()> {
    let cfg = DevflowConfig {
        env: HashMap::from([
            ("DATABASE_URL".into(), "string".into()),
            ("PORT".into(), "int".into()),
        ]),
        services: vec![ServiceDef {
            name: "app".into(),
            command: "cargo run".into(),
        }],
        start_commands: vec!["docker compose up -d".into()],
        test_command: Some("cargo test".into()),
        ignore_globs: vec!["target/**".into(), "node_modules/**".into()],
        desired_ports: vec![3000, 5432],
    };
    let content = serde_yaml::to_string(&cfg)?;
    fs::write(root.join(".devflow.yaml"), content)?;
    Ok(())
}
