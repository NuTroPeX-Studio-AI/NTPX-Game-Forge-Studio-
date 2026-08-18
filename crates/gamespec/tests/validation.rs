use gamespec::GameSpec;

const SAMPLE: &str = include_str!("../../../examples/signal-born-first-thought/gamespec.json");

fn sample() -> GameSpec {
    serde_json::from_str(SAMPLE).expect("sample must deserialize")
}

#[test]
fn rejects_unsupported_schema_versions() {
    let mut spec = sample();
    spec.schema_version = "99.0.0".to_owned();

    let report = spec.validate();
    assert!(!report.is_valid());
    assert!(report
        .issues()
        .iter()
        .any(|issue| issue.code == "unsupported_schema_version"));
}

#[test]
fn rejects_duplicate_global_ids() {
    let mut spec = sample();
    spec.entities[0].id = spec.components[0].id.clone();

    let report = spec.validate();
    assert!(!report.is_valid());
    assert!(report.issues().iter().any(|issue| issue.code == "duplicate_id"));
}

#[test]
fn rejects_unresolved_component_references() {
    let mut value: serde_json::Value = serde_json::from_str(SAMPLE).expect("sample must be JSON");
    value["entities"][0]["components"][0] = serde_json::Value::String("missing-component".to_owned());
    let spec: GameSpec = serde_json::from_value(value).expect("mutated sample must deserialize");

    let report = spec.validate();
    assert!(!report.is_valid());
    assert!(report
        .issues()
        .iter()
        .any(|issue| issue.code == "missing_component_reference"));
}
