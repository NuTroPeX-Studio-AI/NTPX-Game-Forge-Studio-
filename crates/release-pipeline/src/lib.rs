//! Cross-platform release manifest and human-promotion gates for NTPX GameForge Ω∞.
//!
//! This crate validates release evidence. It does not sign, notarize, upload, publish, or promote.

use std::{collections::BTreeSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageTarget {
    Windows,
    Linux,
    Macos,
    Android,
    Ios,
    SteamDeck,
    ConsoleAdapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
    Staging,
    Canary,
    Production,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEvidence {
    pub target: PackageTarget,
    pub artifact_path: String,
    pub artifact_sha256: String,
    pub sbom_path: String,
    pub provenance_attestation_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub version: String,
    pub changelog_path: String,
    pub dependency_audit_passed: bool,
    pub crash_symbols_path: String,
    pub packages: Vec<PackageEvidence>,
    pub rollback_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionGate {
    EligibleForHumanApproval,
    Blocked { reasons: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseError {
    EmptyVersion,
    EmptyChangelog,
    EmptyCrashSymbols,
    EmptyRollbackReference,
    DuplicateTarget(PackageTarget),
    EmptyArtifactPath,
    InvalidArtifactDigest,
    EmptySbom,
    EmptyProvenance,
}

impl fmt::Display for ReleaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "release validation failed: {self:?}")
    }
}

impl Error for ReleaseError {}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Validate evidence and determine whether a release may be presented for human approval.
///
/// Production never auto-promotes: success means eligible for explicit human approval only.
///
/// # Errors
/// Returns [`ReleaseError`] for malformed or duplicate package evidence.
pub fn evaluate_release(
    manifest: &ReleaseManifest,
    required_targets: &BTreeSet<PackageTarget>,
    channel: ReleaseChannel,
) -> Result<PromotionGate, ReleaseError> {
    if manifest.version.trim().is_empty() {
        return Err(ReleaseError::EmptyVersion);
    }
    if manifest.changelog_path.trim().is_empty() {
        return Err(ReleaseError::EmptyChangelog);
    }
    if manifest.crash_symbols_path.trim().is_empty() {
        return Err(ReleaseError::EmptyCrashSymbols);
    }
    if manifest.rollback_reference.trim().is_empty() {
        return Err(ReleaseError::EmptyRollbackReference);
    }
    let mut targets = BTreeSet::new();
    for package in &manifest.packages {
        if !targets.insert(package.target) {
            return Err(ReleaseError::DuplicateTarget(package.target));
        }
        if package.artifact_path.trim().is_empty() {
            return Err(ReleaseError::EmptyArtifactPath);
        }
        if !valid_sha256(&package.artifact_sha256) {
            return Err(ReleaseError::InvalidArtifactDigest);
        }
        if package.sbom_path.trim().is_empty() {
            return Err(ReleaseError::EmptySbom);
        }
        if package.provenance_attestation_path.trim().is_empty() {
            return Err(ReleaseError::EmptyProvenance);
        }
    }

    let mut reasons = Vec::new();
    if !manifest.dependency_audit_passed {
        reasons.push("dependency audit did not pass".to_owned());
    }
    for missing in required_targets.difference(&targets) {
        reasons.push(format!("missing package target: {missing:?}"));
    }
    if matches!(channel, ReleaseChannel::Production) {
        reasons.push("production promotion requires explicit human approval".to_owned());
    }
    if reasons.is_empty() {
        Ok(PromotionGate::EligibleForHumanApproval)
    } else {
        reasons.sort();
        Ok(PromotionGate::Blocked { reasons })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> ReleaseManifest {
        ReleaseManifest {
            version: "0.1.0-alpha.1".to_owned(),
            changelog_path: "CHANGELOG.md".to_owned(),
            dependency_audit_passed: true,
            crash_symbols_path: "dist/symbols".to_owned(),
            packages: vec![PackageEvidence {
                target: PackageTarget::Windows,
                artifact_path: "dist/game.exe".to_owned(),
                artifact_sha256: "a".repeat(64),
                sbom_path: "dist/sbom.json".to_owned(),
                provenance_attestation_path: "dist/provenance.json".to_owned(),
            }],
            rollback_reference: "release:previous".to_owned(),
        }
    }

    #[test]
    fn production_never_auto_promotes() {
        let gate = evaluate_release(
            &manifest(),
            &BTreeSet::from([PackageTarget::Windows]),
            ReleaseChannel::Production,
        )
        .expect("valid evidence");
        assert!(matches!(gate, PromotionGate::Blocked { .. }));
    }

    #[test]
    fn missing_required_target_is_blocked() {
        let gate = evaluate_release(
            &manifest(),
            &BTreeSet::from([PackageTarget::Windows, PackageTarget::Linux]),
            ReleaseChannel::Canary,
        )
        .expect("valid evidence");
        assert!(matches!(gate, PromotionGate::Blocked { .. }));
    }
}
