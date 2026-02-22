use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{env, path::Path, sync::mpsc::channel, time::Duration};
use tokio::process::Command;

use crate::utils::{config::load_config, language::{detect_project_language, Language}};

pub async fn run() -> Result<()> {
    let root = env::current_dir()?;
    let language = detect_project_language(&root);
    let cfg = load_config(&root).unwrap_or_default();

    let mut builder = GlobSetBuilder::new();
    for g in &cfg.ignore_globs {
        let _ = builder.add(Glob::new(g)?);
    }
    let ignore_set = builder.build()?;

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(&root, RecursiveMode::Recursive)?;

    println!("watching for changes...");

    loop {
        if let Ok(event) = rx.recv_timeout(Duration::from_secs(1)) {
            let Ok(ev) = event else { continue; };
            let impacted = ev
                .paths
                .iter()
                .filter(|p| !is_ignored(p, &ignore_set, &root))
                .collect::<Vec<_>>();
            if impacted.is_empty() {
                continue;
            }
            println!("changed files: {}", impacted.len());
            run_impacted_tests(language).await?;
        }
    }
}

fn is_ignored(path: &Path, set: &globset::GlobSet, root: &Path) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path);
    set.is_match(rel)
}

async fn run_impacted_tests(language: Language) -> Result<()> {
    let mut cmd = match language {
        Language::Python => {
            let mut c = Command::new("pytest");
            c.arg("-q");
            c
        }
        Language::Node => {
            let mut c = Command::new("npx");
            c.args(["jest", "--passWithNoTests"]);
            c
        }
        Language::Rust => {
            let mut c = Command::new("cargo");
            c.arg("test");
            c
        }
        Language::Go => {
            let mut c = Command::new("go");
            c.args(["test", "./..."]);
            c
        }
        Language::Unknown => return Ok(()),
    };

    let status = cmd.status().await?;
    println!("test run status: {}", status);
    Ok(())
}
