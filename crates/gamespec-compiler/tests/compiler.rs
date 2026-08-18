use gamespec::from_json;
use gamespec_compiler::compile_manifest;

const SAMPLE: &str = include_str!("../../../examples/signal-born-first-thought/game.spec.json");

#[test]
fn compilation_manifest_is_deterministic() {
    let spec = from_json(SAMPLE).expect("sample must validate");
    let first = compile_manifest(&spec).expect("compile should succeed");
    let second = compile_manifest(&spec).expect("compile should repeat");
    assert_eq!(first, second);
    assert!(first.dependency_graph.is_acyclic());
}
