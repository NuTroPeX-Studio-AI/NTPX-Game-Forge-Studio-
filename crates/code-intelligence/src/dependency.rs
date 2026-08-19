//! Deterministic advisory dependency graph derived from the Phase 06 lexical reference graph.
//!
//! The graph intentionally models repository-file-to-symbol dependency evidence. It does not claim
//! AST, compiler, linker, call-graph, or language-server semantics and grants no source-mutation or
//! execution authority.

use serde::{Deserialize, Serialize};

use crate::{build_reference_graph, sha256_hex, CodeIntelligenceError, ReferenceResolution, RepositoryIndex};

/// One deterministic resolved dependency edge from an indexed lexical reference to a symbol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from_path: String,
    pub reference_line: u32,
    pub target: String,
    pub target_symbol_id: String,
    pub target_symbol_path: String,
    pub target_symbol_line: u32,
}

/// Deterministic repository-level advisory dependency graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub edges: Vec<DependencyEdge>,
    pub ambiguous_reference_count: u64,
    pub unresolved_reference_count: u64,
    pub evidence_sha256: String,
}

/// Build deterministic dependency edges from the repository's lexical reference graph.
///
/// Only uniquely resolved references become dependency edges. Ambiguous and unresolved references
/// remain visible through explicit counts and are never promoted into guessed edges.
///
/// # Errors
/// Returns [`CodeIntelligenceError`] if reference-graph or dependency-evidence serialization fails.
pub fn build_dependency_graph(
    index: &RepositoryIndex,
) -> Result<DependencyGraph, CodeIntelligenceError> {
    let reference_graph = build_reference_graph(index)?;
    let mut edges = Vec::new();
    let mut ambiguous_reference_count = 0_u64;
    let mut unresolved_reference_count = 0_u64;

    for reference in reference_graph.references {
        match reference.resolution {
            ReferenceResolution::Resolved {
                symbol_id,
                symbol_path,
                symbol_line,
            } => edges.push(DependencyEdge {
                from_path: reference.from_path,
                reference_line: reference.line,
                target: reference.target,
                target_symbol_id: symbol_id,
                target_symbol_path: symbol_path,
                target_symbol_line: symbol_line,
            }),
            ReferenceResolution::Ambiguous { .. } => {
                ambiguous_reference_count = ambiguous_reference_count.saturating_add(1);
            }
            ReferenceResolution::Unresolved => {
                unresolved_reference_count = unresolved_reference_count.saturating_add(1);
            }
        }
    }

    edges.sort_by(|left, right| {
        (
            &left.from_path,
            left.reference_line,
            &left.target,
            &left.target_symbol_id,
        )
            .cmp(&(
                &right.from_path,
                right.reference_line,
                &right.target,
                &right.target_symbol_id,
            ))
    });

    let evidence_payload = (&edges, ambiguous_reference_count, unresolved_reference_count);
    let encoded = serde_json::to_vec(&evidence_payload).map_err(CodeIntelligenceError::Encode)?;
    let evidence_sha256 = sha256_hex(&encoded);

    Ok(DependencyGraph {
        edges,
        ambiguous_reference_count,
        unresolved_reference_count,
        evidence_sha256,
    })
}

#[cfg(test)]
mod tests {
    use crate::{build_repository_index, Language, LexicalLanguageDriver, SourceFile};

    use super::*;

    #[test]
    fn uniquely_resolved_reference_becomes_dependency_edge() {
        let rust = LexicalLanguageDriver::rust();
        let sources = vec![
            SourceFile {
                path: "src/core.rs".to_owned(),
                language: Language::Rust,
                content: "struct Core {}".to_owned(),
                generated: false,
            },
            SourceFile {
                path: "src/consumer.rs".to_owned(),
                language: Language::Rust,
                content: "use Core;\nfn consume() {}".to_owned(),
                generated: false,
            },
        ];
        let index = build_repository_index(&sources, &[&rust]).expect("valid index");
        let graph = build_dependency_graph(&index).expect("valid dependency graph");

        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from_path, "src/consumer.rs");
        assert_eq!(graph.edges[0].target_symbol_path, "src/core.rs");
        assert_eq!(graph.ambiguous_reference_count, 0);
        assert_eq!(graph.unresolved_reference_count, 0);
    }

    #[test]
    fn ambiguous_and_unresolved_references_never_become_guessed_edges() {
        let rust = LexicalLanguageDriver::rust();
        let sources = vec![
            SourceFile {
                path: "src/a.rs".to_owned(),
                language: Language::Rust,
                content: "struct Shared {}".to_owned(),
                generated: false,
            },
            SourceFile {
                path: "src/b.rs".to_owned(),
                language: Language::Rust,
                content: "struct Shared {}".to_owned(),
                generated: false,
            },
            SourceFile {
                path: "src/consumer.rs".to_owned(),
                language: Language::Rust,
                content: "use Shared;\nuse Missing;".to_owned(),
                generated: false,
            },
        ];
        let index = build_repository_index(&sources, &[&rust]).expect("valid index");
        let graph = build_dependency_graph(&index).expect("valid dependency graph");

        assert!(graph.edges.is_empty());
        assert_eq!(graph.ambiguous_reference_count, 1);
        assert_eq!(graph.unresolved_reference_count, 1);
    }

    #[test]
    fn equivalent_source_order_produces_same_dependency_evidence() {
        let rust = LexicalLanguageDriver::rust();
        let first = vec![
            SourceFile {
                path: "src/core.rs".to_owned(),
                language: Language::Rust,
                content: "struct Core {}".to_owned(),
                generated: false,
            },
            SourceFile {
                path: "src/consumer.rs".to_owned(),
                language: Language::Rust,
                content: "use Core;".to_owned(),
                generated: false,
            },
        ];
        let mut second = first.clone();
        second.reverse();

        let first_index = build_repository_index(&first, &[&rust]).expect("valid index");
        let second_index = build_repository_index(&second, &[&rust]).expect("valid index");
        let first_graph = build_dependency_graph(&first_index).expect("valid dependency graph");
        let second_graph = build_dependency_graph(&second_index).expect("valid dependency graph");

        assert_eq!(first_graph, second_graph);
    }
}
