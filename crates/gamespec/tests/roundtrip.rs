use gamespec::{from_json, migrate_json, to_pretty_json, DependencyGraph, GameSpec, GameSpecError};

const SAMPLE: &str = include_str!("../../../examples/signal-born-first-thought/game.spec.json");
const LEGACY: &str = include_str!("../../../examples/signal-born-first-thought/legacy-0.1.spec.json");

#[test]
fn signal_born_sample_round_trips_deterministically() {
    let spec = from_json(SAMPLE).expect("sample GameSpec should validate");
    let first = to_pretty_json(&spec).expect("sample should serialize");
    let reparsed = from_json(&first).expect("serialized GameSpec should validate");
    let second = to_pretty_json(&reparsed).expect("reparsed sample should serialize");
    assert_eq!(spec, reparsed);
    assert_eq!(first, second);
}

#[test]
fn full_sample_graph_is_acyclic() {
    let spec = from_json(SAMPLE).expect("sample GameSpec should validate");
    let graph = DependencyGraph::from_spec(&spec);
    assert!(graph.is_acyclic());
    assert!(!graph.edges.is_empty());
}

#[test]
fn duplicate_ids_are_rejected() {
    let mut value: serde_json::Value = serde_json::from_str(SAMPLE).expect("sample must be JSON");
    value["entities"][0]["id"] = value["components"][0]["id"].clone();
    let invalid = serde_json::to_string(&value).expect("mutated sample should encode");
    let error = from_json(&invalid).expect_err("duplicate IDs must fail");
    assert!(matches!(error, GameSpecError::Validation(_)));
}

#[test]
fn unresolved_references_are_rejected() {
    let mut value: serde_json::Value = serde_json::from_str(SAMPLE).expect("sample must be JSON");
    value["entities"][0]["components"][0] = serde_json::Value::String("missing-component".to_owned());
    let invalid = serde_json::to_string(&value).expect("mutated sample should encode");
    assert!(matches!(from_json(&invalid), Err(GameSpecError::Validation(_))));
}

#[test]
fn dependency_cycles_are_rejected() {
    let mut value: serde_json::Value = serde_json::from_str(SAMPLE).expect("sample must be JSON");
    value["architecture"]["modules"][0]["depends_on"] = serde_json::json!(["monad-system"]);
    let invalid = serde_json::to_string(&value).expect("mutated sample should encode");
    let error = from_json(&invalid).expect_err("cycle must fail");
    match error {
        GameSpecError::Validation(errors) => assert!(errors.iter().any(|error| error.code == "dependency_cycle")),
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn legacy_fixture_migrates_to_current_schema() {
    let migrated = migrate_json(LEGACY).expect("legacy fixture should migrate");
    let spec: GameSpec = from_json(&migrated).expect("migrated fixture should validate");
    assert_eq!(spec.schema_version, gamespec::CURRENT_SCHEMA_VERSION);
    assert_eq!(spec.project.id.as_str(), "signal-born-first-thought");
    assert_eq!(spec.components.len(), 2);
}

#[test]
fn checked_in_schema_is_valid_json() {
    let schema = gamespec::json_schema().expect("schema must parse");
    assert_eq!(schema["$id"], "https://ntpx.dev/schemas/gamespec/0.1.0");
}
