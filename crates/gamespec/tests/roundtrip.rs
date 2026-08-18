use gamespec::GameSpec;

const SAMPLE: &str = include_str!("../../../examples/signal-born-first-thought/gamespec.json");
const SCHEMA: &str = include_str!("../../../schemas/gamespec/v0.1.0/gamespec.schema.json");

#[test]
fn signal_born_sample_round_trips_and_validates() {
    let parsed: GameSpec = serde_json::from_str(SAMPLE).expect("sample must deserialize");
    assert_eq!(parsed.project.title, "Signal Born: First Thought");
    assert!(parsed.validate().is_valid(), "sample must validate");

    let encoded = serde_json::to_string_pretty(&parsed).expect("sample must serialize");
    let reparsed: GameSpec = serde_json::from_str(&encoded).expect("round trip must deserialize");
    assert_eq!(parsed, reparsed);
}

#[test]
fn checked_in_json_schema_is_valid_json() {
    let schema: serde_json::Value = serde_json::from_str(SCHEMA).expect("schema must be valid JSON");
    assert_eq!(schema["$id"], "https://ntpx.dev/schemas/gamespec/0.1.0");
}
