//! Controlled self-improvement proposal and promotion gates for NTPX GameForge Ω∞.
//!
//! This crate evaluates evidence only. It cannot mutate policy, merge code, or promote a release.

use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub suite_id: String,
    pub score_basis_points: u16,
    pub security_passed: bool,
    pub regression_passed: bool,
    pub performance_passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovementProposal {
    pub id: String,
    pub objective: String,
    pub changed_paths: Vec<String>,
    pub rollback_plan: String,
    pub requires_policy_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionDecision {
    EligibleForHumanReview { score_delta: i32 },
    Reject { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImprovementError {
    EmptyProposalId,
    EmptyObjective,
    EmptyChangedPaths,
    EmptyRollbackPlan,
    SuiteMismatch,
    InvalidScore,
}

impl fmt::Display for ImprovementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "controlled improvement validation failed: {self:?}"
        )
    }
}

impl Error for ImprovementError {}

/// Evaluate a sandboxed candidate against its baseline.
///
/// Passing all gates only makes the candidate eligible for human review. It never auto-promotes.
///
/// # Errors
/// Returns [`ImprovementError`] for malformed proposals or incomparable benchmark evidence.
pub fn evaluate_candidate(
    proposal: &ImprovementProposal,
    baseline: &BenchmarkResult,
    candidate: &BenchmarkResult,
) -> Result<PromotionDecision, ImprovementError> {
    if proposal.id.trim().is_empty() {
        return Err(ImprovementError::EmptyProposalId);
    }
    if proposal.objective.trim().is_empty() {
        return Err(ImprovementError::EmptyObjective);
    }
    if proposal.changed_paths.is_empty() {
        return Err(ImprovementError::EmptyChangedPaths);
    }
    if proposal.rollback_plan.trim().is_empty() {
        return Err(ImprovementError::EmptyRollbackPlan);
    }
    if baseline.suite_id != candidate.suite_id {
        return Err(ImprovementError::SuiteMismatch);
    }
    if baseline.score_basis_points > 10_000 || candidate.score_basis_points > 10_000 {
        return Err(ImprovementError::InvalidScore);
    }
    if !candidate.security_passed {
        return Ok(PromotionDecision::Reject {
            reason: "security gate failed".to_owned(),
        });
    }
    if !candidate.regression_passed {
        return Ok(PromotionDecision::Reject {
            reason: "regression gate failed".to_owned(),
        });
    }
    if !candidate.performance_passed {
        return Ok(PromotionDecision::Reject {
            reason: "performance gate failed".to_owned(),
        });
    }
    let score_delta =
        i32::from(candidate.score_basis_points) - i32::from(baseline.score_basis_points);
    if score_delta < 0 {
        return Ok(PromotionDecision::Reject {
            reason: "candidate benchmark regressed".to_owned(),
        });
    }
    Ok(PromotionDecision::EligibleForHumanReview { score_delta })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passing_candidate_still_requires_human_review() {
        let proposal = ImprovementProposal {
            id: "p1".to_owned(),
            objective: "reduce build failures".to_owned(),
            changed_paths: vec!["src/lib.rs".to_owned()],
            rollback_plan: "revert commit".to_owned(),
            requires_policy_change: false,
        };
        let baseline = BenchmarkResult {
            suite_id: "suite".to_owned(),
            score_basis_points: 8_000,
            security_passed: true,
            regression_passed: true,
            performance_passed: true,
        };
        let mut candidate = baseline.clone();
        candidate.score_basis_points = 8_500;
        assert!(matches!(
            evaluate_candidate(&proposal, &baseline, &candidate).expect("valid evidence"),
            PromotionDecision::EligibleForHumanReview { .. }
        ));
    }
}
