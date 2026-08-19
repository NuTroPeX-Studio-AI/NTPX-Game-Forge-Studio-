//! Deterministic exact-edit planning boundary for NTPX `GameForge` Ω∞.
//!
//! This crate binds a proposed whole-file replacement to exact repository and source evidence.
//! It performs no source mutation and grants no filesystem, repository, tool, engine, credential,
//! publishing, signing, or release authority.

use std::{error::Error, fmt};

use code_intelligence::{Language, RepositoryIndex, SourceFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Deterministic evidence-bound whole-file replacement plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactEditPlan {
    pub path: String,
    pub language: Language,
    pub expected_content_sha256: String,
    pub replacement_content_sha256: String,
    pub replacement_content: String,
    pub repository_evidence_sha256: String,
    pub evidence_sha256: String,
}

/// Fail-closed exact-edit planning and freshness errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditPlanError {
    EmptyPath,
    MissingIndexedFile(String),
    GeneratedSource(String),
    PathMismatch {
        expected: String,
        actual: String,
    },
    LanguageMismatch {
        expected: Language,
        actual: Language,
    },
    GeneratedStateMismatch {
        expected: bool,
        actual: bool,
    },
    StaleSource {
        expected: String,
        actual: String,
    },
    RepositoryEvidenceMismatch {
        expected: String,
        actual: String,
    },
    NoChange(String),
}

impl fmt::Display for EditPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => formatter.write_str("edit source path must not be empty"),
            Self::MissingIndexedFile(path) => {
                write!(
                    formatter,
                    "source '{path}' is not present in the repository index"
                )
            }
            Self::GeneratedSource(path) => {
                write!(
                    formatter,
                    "refusing exact edit plan for generated source '{path}'"
                )
            }
            Self::PathMismatch { expected, actual } => write!(
                formatter,
                "edit plan path mismatch: expected '{expected}', got '{actual}'"
            ),
            Self::LanguageMismatch { expected, actual } => write!(
                formatter,
                "edit plan language mismatch: expected {expected:?}, got {actual:?}"
            ),
            Self::GeneratedStateMismatch { expected, actual } => write!(
                formatter,
                "edit plan generated-state mismatch: expected {expected}, got {actual}"
            ),
            Self::StaleSource { expected, actual } => write!(
                formatter,
                "edit plan source digest is stale: expected {expected}, got {actual}"
            ),
            Self::RepositoryEvidenceMismatch { expected, actual } => write!(
                formatter,
                "edit plan repository evidence mismatch: expected {expected}, got {actual}"
            ),
            Self::NoChange(path) => {
                write!(
                    formatter,
                    "replacement for '{path}' does not change source content"
                )
            }
        }
    }
}

impl Error for EditPlanError {}

/// Build a deterministic whole-file replacement plan bound to exact indexed source evidence.
///
/// The source must exactly match indexed language, generated-state, and SHA-256 content evidence.
/// Generated sources are rejected in this increment. The returned plan is advisory data only.
///
/// # Errors
/// Returns [`EditPlanError`] when source/index evidence differs, the source is generated, or the
/// replacement is an exact no-op.
pub fn plan_exact_file_replacement(
    index: &RepositoryIndex,
    source: &SourceFile,
    replacement_content: impl Into<String>,
) -> Result<ExactEditPlan, EditPlanError> {
    if source.path.trim().is_empty() {
        return Err(EditPlanError::EmptyPath);
    }
    let indexed = index
        .files
        .get(&source.path)
        .ok_or_else(|| EditPlanError::MissingIndexedFile(source.path.clone()))?;
    if indexed.language != source.language {
        return Err(EditPlanError::LanguageMismatch {
            expected: indexed.language,
            actual: source.language,
        });
    }
    if indexed.generated != source.generated {
        return Err(EditPlanError::GeneratedStateMismatch {
            expected: indexed.generated,
            actual: source.generated,
        });
    }
    if source.generated {
        return Err(EditPlanError::GeneratedSource(source.path.clone()));
    }

    let current_digest = sha256_hex(source.content.as_bytes());
    if indexed.content_sha256 != current_digest {
        return Err(EditPlanError::StaleSource {
            expected: indexed.content_sha256.clone(),
            actual: current_digest,
        });
    }

    let replacement_content = replacement_content.into();
    let replacement_content_sha256 = sha256_hex(replacement_content.as_bytes());
    if replacement_content_sha256 == indexed.content_sha256 {
        return Err(EditPlanError::NoChange(source.path.clone()));
    }

    let mut plan = ExactEditPlan {
        path: source.path.clone(),
        language: source.language,
        expected_content_sha256: indexed.content_sha256.clone(),
        replacement_content_sha256,
        replacement_content,
        repository_evidence_sha256: index.evidence_sha256.clone(),
        evidence_sha256: String::new(),
    };
    plan.evidence_sha256 = plan_digest(&plan);
    Ok(plan)
}

/// Re-check a plan against exact repository evidence and current source before mutation.
///
/// This function performs no write. A later Governor-bound mutation adapter can use it to fail
/// closed if repository or source state changes after planning.
///
/// # Errors
/// Returns [`EditPlanError`] for repository evidence drift or path/language/generated/content
/// mismatch between the plan and current source.
pub fn verify_edit_plan_fresh(
    plan: &ExactEditPlan,
    repository_evidence_sha256: &str,
    current_source: &SourceFile,
) -> Result<(), EditPlanError> {
    if plan.repository_evidence_sha256 != repository_evidence_sha256 {
        return Err(EditPlanError::RepositoryEvidenceMismatch {
            expected: plan.repository_evidence_sha256.clone(),
            actual: repository_evidence_sha256.to_owned(),
        });
    }
    if plan.path != current_source.path {
        return Err(EditPlanError::PathMismatch {
            expected: plan.path.clone(),
            actual: current_source.path.clone(),
        });
    }
    if plan.language != current_source.language {
        return Err(EditPlanError::LanguageMismatch {
            expected: plan.language,
            actual: current_source.language,
        });
    }
    if current_source.generated {
        return Err(EditPlanError::GeneratedSource(current_source.path.clone()));
    }

    let actual = sha256_hex(current_source.content.as_bytes());
    if plan.expected_content_sha256 != actual {
        return Err(EditPlanError::StaleSource {
            expected: plan.expected_content_sha256.clone(),
            actual,
        });
    }
    Ok(())
}

fn plan_digest(plan: &ExactEditPlan) -> String {
    let canonical = format!(
        "{}\n{:?}\n{}\n{}\n{}\n{}",
        plan.path,
        plan.language,
        plan.expected_content_sha256,
        plan.replacement_content_sha256,
        plan.repository_evidence_sha256,
        plan.replacement_content
    );
    sha256_hex(canonical.as_bytes())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use code_intelligence::{build_repository_index, LexicalLanguageDriver};

    fn source(content: &str, generated: bool) -> SourceFile {
        SourceFile {
            path: "src/player.rs".to_owned(),
            language: Language::Rust,
            content: content.to_owned(),
            generated,
        }
    }

    fn index_for(source: &SourceFile) -> RepositoryIndex {
        let rust = LexicalLanguageDriver::rust();
        build_repository_index(std::slice::from_ref(source), &[&rust]).expect("valid index")
    }

    #[test]
    fn equivalent_inputs_produce_identical_plan_evidence() {
        let current = source("fn player() {}\n", false);
        let index = index_for(&current);
        let first = plan_exact_file_replacement(&index, &current, "fn player() { tick(); }\n")
            .expect("valid edit plan");
        let second = plan_exact_file_replacement(&index, &current, "fn player() { tick(); }\n")
            .expect("valid edit plan");

        assert_eq!(first, second);
        assert_ne!(
            first.expected_content_sha256,
            first.replacement_content_sha256
        );
        assert_eq!(first.repository_evidence_sha256, index.evidence_sha256);
    }

    #[test]
    fn stale_source_is_rejected_during_planning() {
        let indexed_source = source("fn player() {}\n", false);
        let index = index_for(&indexed_source);
        let changed_source = source("fn player() { moved(); }\n", false);

        let error =
            plan_exact_file_replacement(&index, &changed_source, "fn player() { tick(); }\n")
                .expect_err("stale source must fail");
        assert!(matches!(error, EditPlanError::StaleSource { .. }));
    }

    #[test]
    fn freshness_check_rejects_source_changed_after_planning() {
        let current = source("fn player() {}\n", false);
        let index = index_for(&current);
        let plan = plan_exact_file_replacement(&index, &current, "fn player() { tick(); }\n")
            .expect("valid edit plan");
        let changed = source("fn player() { changed_elsewhere(); }\n", false);

        let error = verify_edit_plan_fresh(&plan, &index.evidence_sha256, &changed)
            .expect_err("post-plan source drift must fail");
        assert!(matches!(error, EditPlanError::StaleSource { .. }));
    }

    #[test]
    fn freshness_check_rejects_repository_evidence_drift() {
        let current = source("fn player() {}\n", false);
        let index = index_for(&current);
        let plan = plan_exact_file_replacement(&index, &current, "fn player() { tick(); }\n")
            .expect("valid edit plan");

        let error = verify_edit_plan_fresh(&plan, "different-repository-evidence", &current)
            .expect_err("repository evidence drift must fail");
        assert!(matches!(
            error,
            EditPlanError::RepositoryEvidenceMismatch { .. }
        ));
    }

    #[test]
    fn generated_sources_are_rejected() {
        let current = source("fn generated_player() {}\n", true);
        let index = index_for(&current);
        let error =
            plan_exact_file_replacement(&index, &current, "fn generated_player() { changed(); }\n")
                .expect_err("generated source mutation plan must fail");
        assert!(matches!(error, EditPlanError::GeneratedSource(_)));
    }

    #[test]
    fn no_op_replacement_is_rejected() {
        let current = source("fn player() {}\n", false);
        let index = index_for(&current);
        let error = plan_exact_file_replacement(&index, &current, current.content.clone())
            .expect_err("no-op plan must fail");
        assert!(matches!(error, EditPlanError::NoChange(_)));
    }
}
