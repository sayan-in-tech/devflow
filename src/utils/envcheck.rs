use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvIssue {
    pub key: String,
    pub reason: String,
}

pub fn parse_dotenv(root: &Path) -> Result<HashMap<String, String>> {
    let path = root.join(".env");
    let mut vars = HashMap::new();
    if !path.exists() {
        return Ok(vars);
    }
    let content = fs::read_to_string(path)?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || !line.contains('=') {
            continue;
        }
        let mut parts = line.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            vars.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(vars)
}

pub fn validate_env_schema(
    schema: &HashMap<String, String>,
    actual: &HashMap<String, String>,
) -> Vec<EnvIssue> {
    let mut issues = Vec::new();
    for (key, typ) in schema {
        match actual.get(key) {
            None => issues.push(EnvIssue {
                key: key.clone(),
                reason: "missing".into(),
            }),
            Some(value) => {
                if typ == "int" && value.parse::<i64>().is_err() {
                    issues.push(EnvIssue {
                        key: key.clone(),
                        reason: "expected int".into(),
                    });
                }
                if typ == "bool" && value.parse::<bool>().is_err() {
                    issues.push(EnvIssue {
                        key: key.clone(),
                        reason: "expected bool".into(),
                    });
                }
            }
        }
    }
    issues
}

pub fn doctor_path_issues() -> Vec<String> {
    let mut issues = Vec::new();
    if env::var_os("PATH").is_none() {
        issues.push("PATH is unset".into());
    }
    if which::which("python").is_err() && which::which("python3").is_err() {
        issues.push("Python not found in PATH".into());
    }
    if which::which("node").is_err() {
        issues.push("Node not found in PATH".into());
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn validates_int_type() {
        let mut schema = HashMap::new();
        schema.insert("PORT".to_string(), "int".to_string());
        let mut actual = HashMap::new();
        actual.insert("PORT".to_string(), "abc".to_string());
        let issues = validate_env_schema(&schema, &actual);
        assert_eq!(issues.len(), 1);
    }
}
