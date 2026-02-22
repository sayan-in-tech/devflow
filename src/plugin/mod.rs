use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Stdio};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    pub command: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub ok: bool,
    pub message: String,
    pub data: serde_json::Value,
}

pub async fn dispatch(name: &str, payload: serde_json::Value) -> Result<PluginResponse> {
    let req = PluginRequest {
        command: name.to_string(),
        payload,
    };

    if name.ends_with(".wasm") {
        bail!("WASM plugin runtime not enabled in this build");
    }

    let executable = resolve_executable_plugin(name)?;
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to launch plugin")?;

    if let Some(stdin) = child.stdin.take() {
        let bytes = serde_json::to_vec(&req)?;
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let mut s = stdin;
            let _ = s.write_all(&bytes).await;
        });
    }

    let output = child.wait_with_output().await?;
    if !output.status.success() {
        bail!("plugin exited with status {}", output.status);
    }

    let resp: PluginResponse = serde_json::from_slice(&output.stdout)
        .context("plugin produced invalid JSON")?;
    Ok(resp)
}

fn resolve_executable_plugin(name: &str) -> Result<PathBuf> {
    let prefixed = if name.starts_with("devflow-plugin-") {
        name.to_string()
    } else {
        format!("devflow-plugin-{name}")
    };

    if let Ok(path) = which::which(&prefixed) {
        return Ok(path);
    }

    let local = PathBuf::from("plugins").join(&prefixed);
    if local.exists() {
        return Ok(local);
    }

    bail!("plugin not found: {prefixed}")
}
