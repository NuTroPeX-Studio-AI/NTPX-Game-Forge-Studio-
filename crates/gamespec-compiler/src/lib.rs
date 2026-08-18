use gamespec::{
    deterministic_id, to_pretty_json, DependencyGraph, GameSpec, GameSpecError, SpecId,
};
use serde::{Deserialize, Serialize};

/// Deterministic compiler output describing a validated specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilationManifest {
    pub schema_version: String,
    pub project_id: SpecId,
    pub artifact_id: SpecId,
    pub dependency_graph: DependencyGraph,
}

/// Compile a validated `GameSpec` into a deterministic manifest.
///
/// # Errors
/// Returns [`GameSpecError`] when the input fails validation or deterministic serialization.
pub fn compile_manifest(spec: &GameSpec) -> Result<CompilationManifest, GameSpecError> {
    let canonical = to_pretty_json(spec)?;
    Ok(CompilationManifest {
        schema_version: spec.schema_version.clone(),
        project_id: spec.project.id.clone(),
        artifact_id: deterministic_id("artifact", &canonical),
        dependency_graph: DependencyGraph::from_spec(spec),
    })
}
