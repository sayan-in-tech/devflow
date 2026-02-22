use anyhow::Result;
use std::{env, fs};

use crate::utils::language::{detect_project_language, Language};

pub async fn run() -> Result<()> {
    let root = env::current_dir()?;
    match detect_project_language(&root) {
        Language::Python => python_report(&root),
        Language::Node => node_report(&root),
        Language::Rust => rust_report(&root),
        Language::Go | Language::Unknown => {
            println!("deps analysis not yet available for this project type")
        }
    }
    Ok(())
}

fn python_report(root: &std::path::Path) {
    let req = root.join("requirements.txt");
    let lock = root.join("poetry.lock");
    println!("python deps");
    println!("requirements: {}", req.exists());
    println!("poetry.lock: {}", lock.exists());
    println!("license_risk_summary: unknown (offline mode)");
}

fn node_report(root: &std::path::Path) {
    let pkg = root.join("package.json");
    let lock = root.join("package-lock.json");
    println!("node deps");
    println!("package.json: {}", pkg.exists());
    println!("lock file: {}", lock.exists());
    if let Ok(s) = fs::read_to_string(pkg) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
            let dep_count = v["dependencies"].as_object().map(|m| m.len()).unwrap_or(0);
            println!("declared packages: {}", dep_count);
        }
    }
    println!("outdated packages: run npm outdated for full list");
}

fn rust_report(root: &std::path::Path) {
    let lock = root.join("Cargo.lock");
    println!("rust deps");
    println!("cargo.lock: {}", lock.exists());
    println!("top transitive bloat: run cargo tree -e features -i <crate>");
    println!("license_risk_summary: run cargo deny when available");
}
