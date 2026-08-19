//! Deterministic advisory diagnostics derived from Phase 06 lexical reference evidence.
//!
//! This crate consumes the verified `code-intelligence` reference graph and reports only
//! ambiguity/unresolved-reference diagnostics. It does not parse source, mutate files, invoke
//! models/tools, use network/credentials, or grant execution authority.

use code_intelligence::{ReferenceGraph, ReferenceResolution};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{error::Error, fmt};

/// Stable diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Warning,
}

/// One deterministic advisory code-intelligence diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticRecord {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub path: String,
    pub line: u32,
    pub target: String,
    pub message: String,
    pub candidate_symbol_ids: Vec<String>,
}

/// Deterministic diagnostic report with reproducible evidence digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub diagnostics: Vec<DiagnosticRecord>,
    pub unresolved_count: usize,
    pub ambiguous_count: usize,
    pub evidence_sha256: String,
}

/// Diagnostic evidence construction failure.
#[derive(Debug)]
pub enum DiagnosticError {
    Encode(serde_json::Error),
}

impl fmt::Display for DiagnosticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(error) => write!(formatter, "diagnostic evidence encoding failed: {error}"),
        }
    }
}

impl Error for DiagnosticError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Build deterministic advisory diagnostics from an existing lexical reference graph.
///
/// Resolved references produce no diagnostic. Ambiguous references preserve all candidate symbol
/// IDs in deterministic order; unresolved references remain explicit warnings. No diagnostic is
/// interpreted as compiler/LSP semantic proof.
///
/// # Errors
/// Returns [`DiagnosticError`] if deterministic evidence serialization fails.
pub fn build_diagnostic_report(graph: &ReferenceGraph) -> Result<DiagnosticReport, DiagnosticError> {
    let mut diagnostics = Vec::new();
    let mut unresolved_count = 0;
    let mut ambiguous_count = 0;

    for reference in &graph.references {
        match &reference.resolution {
            ReferenceResolution::Resolved { .. } => {}
            ReferenceResolution::Unresolved => {
                unresolved_count += 1;
                diagnostics.push(DiagnosticRecord {
                    code: "NTPX-CI-UNRESOLVED-REFERENCE".to_owned(),
                    severity: DiagnosticSeverity::Warning,
                    path: reference.from_path.clone(),
                    line: reference.line,
                    target: reference.target.clone(),
                    message: format!(
                        "lexical reference '{}' has no exact indexed symbol match",
                        reference.target
                    ),
                    candidate_symbol_ids: Vec::new(),
                });
            }
            ReferenceResolution::Ambiguous {
                candidate_symbol_ids,
            } => {
                ambiguous_count += 1;
                let mut candidates = candidate_symbol_ids.clone();
                candidates.sort();
                candidates.dedup();
                diagnostics.push(DiagnosticRecord {
                    code: "NTPX-CI-AMBIGUOUS-REFERENCE".to_owned(),
                    severity: DiagnosticSeverity::Warning,
                    path: reference.from_path.clone(),
                    line: reference.line,
                    target: reference.target.clone(),
                    message: format!(
                        "lexical reference '{}' matches multiple indexed symbols",
                        reference.target
                    ),
                    candidate_symbol_ids: candidates,
                });
            }
        }
    }

    diagnostics.sort_by(|left, right| {
        (
            &left.path,
            left.line,
            &left.code,
            &left.target,
            &left.candidate_symbol_ids,
        )
            .cmp(&(
                &right.path,
                right.line,
                &right.code,
                &right.target,
                &right.candidate_symbol_ids,
            ))
    });

    let encoded = serde_json::to_vec(&(unresolved_count, ambiguous_count, &diagnostics))
        .map_err(DiagnosticError::Encode)?;
    let evidence_sha256 = sha256_hex(&encoded);

    Ok(DiagnosticReport {
        diagnostics,
        unresolved_count,
        ambiguous_count,
        evidence_sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use code_intelligence::{ReferenceGraph, ReferenceResolution, ResolvedReferenceRecord};

    fn graph(references: Vec<ResolvedReferenceRecord>) -> ReferenceGraph {
        ReferenceGraph {
            references,
            evidence_sha256: "upstream-evidence".to_owned(),
        }
    }

    #[test]
    fn resolved_references_do_not_emit_diagnostics() {
        let report = build_diagnostic_report(&graph(vec![ResolvedReferenceRecord {
            from_path: "src/lib.rs".to_owned(),
            target: "Core".to_owned(),
            line: 2,
            resolution: ReferenceResolution::Resolved {
                symbol_id: "symbol-1".to_owned(),
                symbol_path: "src/core.rs".to_owned(),
                symbol_line: 1,
            },
        }]))
        .expect("diagnostic report");

        assert!(report.diagnostics.is_empty());
        assert_eq!(report.unresolved_count, 0);
        assert_eq!(report.ambiguous_count, 0);
    }

    #[test]
    fn unresolved_and_ambiguous_references_emit_explicit_warnings() {
        let report = build_diagnostic_report(&graph(vec![
            ResolvedReferenceRecord {
                from_path: "src/z.rs".to_owned(),
                target: "Missing".to_owned(),
                line: 9,
                resolution: ReferenceResolution::Unresolved,
            },
            ResolvedReferenceRecord {
                from_path: "src/a.rs".to_owned(),
                target: "Shared".to_owned(),
                line: 3,
                resolution: ReferenceResolution::Ambiguous {
                    candidate_symbol_ids: vec!["symbol-b".to_owned(), "symbol-a".to_owned()],
                },
            },
        ]))
        .expect("diagnostic report");

        assert_eq!(report.unresolved_count, 1);
        assert_eq!(report.ambiguous_count, 1);
        assert_eq!(report.diagnostics.len(), 2);
        assert_eq!(report.diagnostics[0].path, "src/a.rs");
        assert_eq!(
            report.diagnostics[0].candidate_symbol_ids,
            vec!["symbol-a", "symbol-b"]
        );
        assert_eq!(
            report.diagnostics[1].code,
            "NTPX-CI-UNRESOLVED-REFERENCE"
        );
    }

    #[test]
    fn equivalent_reference_order_produces_identical_evidence() {
        let first = vec![
            ResolvedReferenceRecord {
                from_path: "src/b.rs".to_owned(),
                target: "MissingB".to_owned(),
                line: 4,
                resolution: ReferenceResolution::Unresolved,
            },
            ResolvedReferenceRecord {
                from_path: "src/a.rs".to_owned(),
                target: "MissingA".to_owned(),
                line: 2,
                resolution: ReferenceResolution::Unresolved,
            },
        ];
        let mut second = first.clone();
        second.reverse();

        let first_report = build_diagnostic_report(&graph(first)).expect("first report");
        let second_report = build_diagnostic_report(&graph(second)).expect("second report");

        assert_eq!(first_report, second_report);
    }

    #[test]
    fn duplicate_ambiguous_candidates_are_canonicalized() {
        let report = build_diagnostic_report(&graph(vec![ResolvedReferenceRecord {
            from_path: "src/lib.rs".to_owned(),
            target: "Shared".to_owned(),
            line: 1,
            resolution: ReferenceResolution::Ambiguous {
                candidate_symbol_ids: vec![
                    "symbol-b".to_owned(),
                    "symbol-a".to_owned(),
                    "symbol-a".to_owned(),
                ],
            },
        }]))
        .expect("diagnostic report");

        assert_eq!(
            report.diagnostics[0].candidate_symbol_ids,
            vec!["symbol-a", "symbol-b"]
        );
    }
}
