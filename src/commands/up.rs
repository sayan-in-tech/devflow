use anyhow::Result;
use std::env;

use crate::utils::{
    config::load_config,
    envcheck::{parse_dotenv, validate_env_schema},
    language::{detect_project_language, expected_toolchain_hint, Language},
};

pub async fn run() -> Result<()> {
    let root = env::current_dir()?;
    let language = detect_project_language(&root);

    println!("devflow up status");
    println!("-----------------");
    println!("language: {:?}", language);

    let tool = match language {
        Language::Python => "python",
        Language::Node => "node",
        Language::Go => "go",
        Language::Rust => "rustc",
        Language::Unknown => "",
    };

    if !tool.is_empty() {
        match which::which(tool) {
            Ok(path) => println!("toolchain: ok ({})", path.display()),
            Err(_) => println!("toolchain: missing ({})", tool),
        }
    }

    if let Some(hint) = expected_toolchain_hint(&root) {
        println!("expected version hint: {}", hint);
    }

    if root.join("docker-compose.yml").exists() || root.join("compose.yaml").exists() {
        println!("services: docker-compose file detected");
    } else {
        println!("services: no compose file");
    }

    if root.join(".devflow.yaml").exists() {
        let cfg = load_config(&root)?;
        let dotenv = parse_dotenv(&root)?;
        let issues = validate_env_schema(&cfg.env, &dotenv);
        if issues.is_empty() {
            println!("env: schema matches .env");
        } else {
            println!("env: {} issues", issues.len());
            for issue in issues {
                println!(" - {}: {}", issue.key, issue.reason);
            }
            println!("recommendation: run `devflow env doctor` and `devflow env fix`");
        }
    } else {
        println!("recommendation: run `devflow init` to create .devflow.yaml");
    }

    Ok(())
}
