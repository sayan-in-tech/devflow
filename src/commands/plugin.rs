use anyhow::Result;

use crate::{cli::PluginArgs, plugin};

pub async fn run(args: PluginArgs) -> Result<()> {
    let payload = match args.payload {
        Some(raw) => {
            serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({ "raw": raw }))
        }
        None => serde_json::json!({}),
    };

    let response = plugin::dispatch(&args.name, payload).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
