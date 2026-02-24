use assert_cmd::{cargo::cargo_bin_cmd, prelude::*};
use predicates::str::contains;

#[test]
fn init_creates_config() {
    let td = tempfile::tempdir().expect("tempdir");
    cargo_bin_cmd!("devflow")
        .expect("binary")
        .current_dir(td.path())
        .arg("init")
        .assert()
        .success()
        .stdout(contains("Created .devflow.yaml"));

    assert!(td.path().join(".devflow.yaml").exists());
}

#[test]
fn port_free_outputs_json_list() {
    let td = tempfile::tempdir().expect("tempdir");
    cargo_bin_cmd!("devflow")
        .expect("binary")
        .current_dir(td.path())
        .args(["port", "--free"])
        .assert()
        .success()
        .stdout(contains("["));
}
