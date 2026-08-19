//! Deterministic lexical reference resolution for the Phase 06 code-intelligence substrate.
//!
//! Resolution is intentionally advisory. It correlates already-indexed lexical reference tokens with
//! exact symbol names present in the repository index. It does not claim compiler, AST, linker, or
//! language-server semantics and grants no source-mutation or execution authority.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{sha256_hex, CodeIntelligenceError, RepositoryIndex, SymbolRecord};

/// Outcome for one lexical reference token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum ReferenceResolution {
    /// Exactly one indexed symbol matched the lexical target.
    Resolved {
        symbol_id: String,
        symbol_path: String,
        symbol_line: u32,
    },
    /// More than one indexed symbol matched; no arbitrary winner is selected.
    Ambiguous { candidate_symbol_ids: Vec<String> },
    /// No indexed symbol matched the lexical target.
    Unresolved,
}

/// One deterministic lexical reference-resolution record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedReferenceRecord {
    pub from_path: String,
    pub target: String,
    pub line: u32,
    pub resolution: ReferenceResolution,
}

/// Deterministic repository-level lexical reference graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceGraph {
    pub references: Vec<ResolvedReferenceRecord>,
    pub evidence_sha256: String,
}

/// Resolve repository lexical references against exact indexed symbol names.
///
/// Equivalent repository indexes produce equivalent ordered records and evidence. Ambiguous matches
/// remain explicit rather than being resolved by enumeration order.
///
/// # Errors
/// Returns [`CodeIntelligenceError`] only if deterministic evidence serialization fails.
pub fn build_reference_graph(
    index: &RepositoryIndex,
) -> Result<ReferenceGraph, CodeIntelligenceError> {
    let mut symbols_by_name: BTreeMap<&str, Vec<&SymbolRecord>> = BTreeMap::new();
    for symbol in index.files.values().flat_map(|file| &file.symbols) {
        symbols_by_name
            .entry(&symbol.name)
            .or_default()
            .push(symbol);
    }
    for candidates in symbols_by_name.values_mut() {
        candidates.sort_by(|left, right| left.id.cmp(&right.id));
    }

    let mut references = Vec::new();
    for file in index.files.values() {
        for reference in &file.references {
            let resolution = match symbols_by_name.get(reference.target.as_str()) {
                None => ReferenceResolution::Unresolved,
                Some(candidates) if candidates.len() == 1 => {
                    let symbol = candidates[0];
                    ReferenceResolution::Resolved {
                        symbol_id: symbol.id.clone(),
                        symbol_path: symbol.path.clone(),
                        symbol_line: symbol.line,
                    }
                }
                Some(candidates) => ReferenceResolution::Ambiguous {
                    candidate_symbol_ids: candidates
                        .iter()
                        .map(|symbol| symbol.id.clone())
                        .collect(),
                },
            };
            references.push(ResolvedReferenceRecord {
                from_path: reference.from_path.clone(),
                target: reference.target.clone(),
                line: reference.line,
                resolution,
            });
        }
    }

    references.sort_by(|left, right| {
        (&left.from_path, left.line, &left.target).cmp(&(
            &right.from_path,
            right.line,
            &right.target,
        ))
    });
    let encoded = serde_json::to_vec(&references).map_err(CodeIntelligenceError::Encode)?;
    let evidence_sha256 = sha256_hex(&encoded);

    Ok(ReferenceGraph {
        references,
        evidence_sha256,
    })
}

#[cfg(test)]
mod tests {
    use crate::{build_repository_index, Language, LexicalLanguageDriver, SourceFile};

    use super::*;

    #[test]
    fn unique_exact_symbol_match_resolves() {
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
        let graph = build_reference_graph(&index).expect("valid graph");

        assert_eq!(graph.references.len(), 1);
        assert!(matches!(
            &graph.references[0].resolution,
            ReferenceResolution::Resolved { symbol_path, .. } if symbol_path == "src/core.rs"
        ));
    }

    #[test]
    fn ambiguous_symbol_match_fails_closed_without_arbitrary_winner() {
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
                path: "src/use_shared.rs".to_owned(),
                language: Language::Rust,
                content: "use Shared;".to_owned(),
                generated: false,
            },
        ];
        let index = build_repository_index(&sources, &[&rust]).expect("valid index");
        let graph = build_reference_graph(&index).expect("valid graph");

        let ReferenceResolution::Ambiguous {
            candidate_symbol_ids,
        } = &graph.references[0].resolution
        else {
            panic!("duplicate symbol name must remain ambiguous");
        };
        assert_eq!(candidate_symbol_ids.len(), 2);
        assert!(candidate_symbol_ids
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn unresolved_reference_is_preserved_as_evidence() {
        let rust = LexicalLanguageDriver::rust();
        let source = SourceFile {
            path: "src/lib.rs".to_owned(),
            language: Language::Rust,
            content: "use Missing;\nfn run() {}".to_owned(),
            generated: false,
        };
        let index = build_repository_index(&[source], &[&rust]).expect("valid index");
        let graph = build_reference_graph(&index).expect("valid graph");

        assert_eq!(
            graph.references[0].resolution,
            ReferenceResolution::Unresolved
        );
    }

    #[test]
    fn equivalent_source_order_produces_same_reference_graph_evidence() {
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
        let first_graph = build_reference_graph(&first_index).expect("valid graph");
        let second_graph = build_reference_graph(&second_index).expect("valid graph");

        assert_eq!(first_graph, second_graph);
    }
}
