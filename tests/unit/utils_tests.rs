use devflow::utils::{
    envcheck::validate_env_schema,
    language::{detect_project_language, Language},
    sanitize::redact,
};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn redact_masks_secret_values() {
    let out = redact("password=supersecret");
    assert!(out.contains("password=<redacted>"));
}

#[test]
fn env_schema_detects_missing_key() {
    let schema = HashMap::from([("PORT".to_string(), "int".to_string())]);
    let actual = HashMap::new();
    let issues = validate_env_schema(&schema, &actual);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].key, "PORT");
}

#[test]
fn language_detects_node_repo() {
    let td = tempdir().expect("tempdir");
    std::fs::write(td.path().join("package.json"), "{}").expect("write");
    assert_eq!(detect_project_language(td.path()), Language::Node);
}
