//! Independent multi-review evaluation contracts for NTPX GameForge Ω∞.

use std::{collections::BTreeSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewKind {
    Independent,
    Adversarial,
    Security,
    Quality,
    Performance,
    Gameplay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub evaluator_id: String,
    pub kind: ReviewKind,
    pub criteria_version: String,
    pub score_basis_points: u16,
    pub approve: bool,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouncilPolicy {
    pub required_kinds: BTreeSet<ReviewKind>,
    pub min_approvals: usize,
    pub min_score_basis_points: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CouncilOutcome {
    Approved,
    Rejected { dissent: Vec<String> },
    Abstained { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    EmptyEvaluator,
    EmptyCriteriaVersion,
    EmptyRationale,
    InvalidScore,
    InvalidPolicy,
    DuplicateEvaluator(String),
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyEvaluator => formatter.write_str("evaluator id must not be empty"),
            Self::EmptyCriteriaVersion => formatter.write_str("criteria version must not be empty"),
            Self::EmptyRationale => formatter.write_str("evaluation rationale must not be empty"),
            Self::InvalidScore => formatter.write_str("evaluation score must be <= 10000 basis points"),
            Self::InvalidPolicy => formatter.write_str("evaluation council policy is invalid"),
            Self::DuplicateEvaluator(id) => write!(formatter, "duplicate evaluator '{id}'"),
        }
    }
}

impl Error for EvaluationError {}

/// Resolve independent reviews without silently manufacturing consensus.
///
/// # Errors
/// Returns [`EvaluationError`] for malformed reviews, duplicate evaluators, or invalid policy.
pub fn evaluate_council(
    policy: &CouncilPolicy,
    reviews: &[Evaluation],
) -> Result<CouncilOutcome, EvaluationError> {
    if policy.required_kinds.is_empty()
        || policy.min_approvals == 0
        || policy.min_score_basis_points > 10_000
    {
        return Err(EvaluationError::InvalidPolicy);
    }
    let mut evaluators = BTreeSet::new();
    let mut seen_kinds = BTreeSet::new();
    for review in reviews {
        if review.evaluator_id.trim().is_empty() {
            return Err(EvaluationError::EmptyEvaluator);
        }
        if review.criteria_version.trim().is_empty() {
            return Err(EvaluationError::EmptyCriteriaVersion);
        }
        if review.rationale.trim().is_empty() {
            return Err(EvaluationError::EmptyRationale);
        }
        if review.score_basis_points > 10_000 {
            return Err(EvaluationError::InvalidScore);
        }
        if !evaluators.insert(review.evaluator_id.clone()) {
            return Err(EvaluationError::DuplicateEvaluator(review.evaluator_id.clone()));
        }
        seen_kinds.insert(review.kind);
    }
    if !policy.required_kinds.is_subset(&seen_kinds) {
        return Ok(CouncilOutcome::Abstained {
            reason: "required independent review kinds are missing".to_owned(),
        });
    }
    let approvals = reviews
        .iter()
        .filter(|review| review.approve && review.score_basis_points >= policy.min_score_basis_points)
        .count();
    if approvals >= policy.min_approvals {
        return Ok(CouncilOutcome::Approved);
    }
    let mut dissent: Vec<_> = reviews
        .iter()
        .filter(|review| !review.approve || review.score_basis_points < policy.min_score_basis_points)
        .map(|review| format!("{}: {}", review.evaluator_id, review.rationale))
        .collect();
    dissent.sort();
    Ok(CouncilOutcome::Rejected { dissent })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_required_review_kind_abstains() {
        let policy = CouncilPolicy {
            required_kinds: BTreeSet::from([ReviewKind::Security, ReviewKind::Quality]),
            min_approvals: 2,
            min_score_basis_points: 8_000,
        };
        let result = evaluate_council(
            &policy,
            &[Evaluation {
                evaluator_id: "security".to_owned(),
                kind: ReviewKind::Security,
                criteria_version: "1".to_owned(),
                score_basis_points: 9_000,
                approve: true,
                rationale: "passes".to_owned(),
            }],
        )
        .expect("valid council input");
        assert!(matches!(result, CouncilOutcome::Abstained { .. }));
    }
}
