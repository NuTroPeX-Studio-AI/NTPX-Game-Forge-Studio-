use std::{path::PathBuf, process::Command};

fn sample_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/signal-born-first-thought/game.spec.json")
}

#[test]
fn cli_validates_reference_sample() {
    let output = Command::new(env!("CARGO_BIN_EXE_gamespec-compiler"))
        .arg("validate")
        .arg(sample_path())
        .output()
        .expect("CLI should execute");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("VALID signal-born-first-thought"));
}

#[test]
fn cli_emits_schema() {
    let output = Command::new(env!("CARGO_BIN_EXE_gamespec-compiler"))
        .arg("schema")
        .output()
        .expect("CLI should execute");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("GameSpec 0.1.0"));
}
