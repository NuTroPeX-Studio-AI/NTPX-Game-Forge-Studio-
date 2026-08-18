use gamespec::{from_json, to_pretty_json, GameSpecError, ProjectSpec};

const SAMPLE: &str = include_str!("../../../examples/signal-born-first-thought/game.spec.json");

#[test]
fn signal_born_sample_round_trips_deterministically() {
    let spec = from_json(SAMPLE).expect("sample GameSpec should validate");
    let first = to_pretty_json(&spec).expect("sample should serialize");
    let reparsed: ProjectSpec = from_json(&first).expect("serialized GameSpec should validate");
    let second = to_pretty_json(&reparsed).expect("reparsed sample should serialize");

    assert_eq!(spec, reparsed);
    assert_eq!(first, second);
}

#[test]
fn duplicate_module_ids_are_rejected() {
    let invalid = r#"{
      "schema_version": "0.1",
      "project_id": "duplicate-test",
      "title": "Duplicate Test",
      "modules": [
        {"id":"world","name":"World","features":[]},
        {"id":"world","name":"World Again","features":[]}
      ]
    }"#;

    let error = from_json(invalid).expect_err("duplicate IDs must fail validation");
    match error {
        GameSpecError::Validation(errors) => {
            assert!(errors.iter().any(|error| error.message.contains("duplicate identifier")));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn unsupported_schema_version_is_rejected() {
    let invalid = r#"{
      "schema_version": "9.9",
      "project_id": "future-test",
      "title": "Future Test",
      "modules": []
    }"#;

    assert!(matches!(from_json(invalid), Err(GameSpecError::Validation(_))));
}
