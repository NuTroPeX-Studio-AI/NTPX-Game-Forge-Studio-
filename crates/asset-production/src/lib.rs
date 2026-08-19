//! Asset production job contracts and provenance gates for NTPX GameForge Ω∞.

use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Image,
    Texture,
    Material,
    Mesh,
    Animation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetJob {
    pub id: String,
    pub kind: AssetKind,
    pub generator_adapter: String,
    pub output_format: String,
    pub license: String,
    pub provenance: String,
    pub quality_profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetArtifact {
    pub job_id: String,
    pub path: String,
    pub content_sha256: String,
    pub license: String,
    pub provenance: String,
    pub quality_passed: bool,
}

pub trait AssetGeneratorAdapter: Send + Sync {
    fn adapter_id(&self) -> &str;
    fn supported_kinds(&self) -> &[AssetKind];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetError {
    EmptyJobId,
    EmptyAdapter,
    EmptyFormat,
    EmptyLicense,
    EmptyProvenance,
    EmptyQualityProfile,
    UnsupportedKind,
    EmptyArtifactPath,
    JobMismatch,
    InvalidDigest,
    QualityRejected,
}

impl fmt::Display for AssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "asset production validation failed: {self:?}")
    }
}

impl Error for AssetError {}

/// Validate one asset job against an explicit generator adapter.
///
/// # Errors
/// Returns [`AssetError`] for malformed jobs or unsupported asset kinds.
pub fn validate_job(job: &AssetJob, adapter: &dyn AssetGeneratorAdapter) -> Result<(), AssetError> {
    if job.id.trim().is_empty() {
        return Err(AssetError::EmptyJobId);
    }
    if job.generator_adapter.trim().is_empty() || job.generator_adapter != adapter.adapter_id() {
        return Err(AssetError::EmptyAdapter);
    }
    if job.output_format.trim().is_empty() {
        return Err(AssetError::EmptyFormat);
    }
    if job.license.trim().is_empty() {
        return Err(AssetError::EmptyLicense);
    }
    if job.provenance.trim().is_empty() {
        return Err(AssetError::EmptyProvenance);
    }
    if job.quality_profile.trim().is_empty() {
        return Err(AssetError::EmptyQualityProfile);
    }
    if !adapter.supported_kinds().contains(&job.kind) {
        return Err(AssetError::UnsupportedKind);
    }
    Ok(())
}

/// Validate produced asset evidence before import.
///
/// # Errors
/// Returns [`AssetError`] if artifact identity/provenance/quality evidence is malformed.
pub fn validate_artifact(job: &AssetJob, artifact: &AssetArtifact) -> Result<(), AssetError> {
    if artifact.job_id != job.id {
        return Err(AssetError::JobMismatch);
    }
    if artifact.path.trim().is_empty() {
        return Err(AssetError::EmptyArtifactPath);
    }
    if artifact.content_sha256.len() != 64
        || !artifact
            .content_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(AssetError::InvalidDigest);
    }
    if artifact.license != job.license || artifact.provenance.trim().is_empty() {
        return Err(AssetError::EmptyProvenance);
    }
    if !artifact.quality_passed {
        return Err(AssetError::QualityRejected);
    }
    Ok(())
}

#[must_use]
pub fn artifact_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Local;
    impl AssetGeneratorAdapter for Local {
        fn adapter_id(&self) -> &str {
            "local"
        }
        fn supported_kinds(&self) -> &[AssetKind] {
            &[AssetKind::Image, AssetKind::Texture]
        }
    }

    #[test]
    fn unsupported_asset_kind_fails_closed() {
        let job = AssetJob {
            id: "a1".to_owned(),
            kind: AssetKind::Mesh,
            generator_adapter: "local".to_owned(),
            output_format: "glb".to_owned(),
            license: "project-owned".to_owned(),
            provenance: "prompt:1".to_owned(),
            quality_profile: "game-ready".to_owned(),
        };
        assert_eq!(validate_job(&job, &Local), Err(AssetError::UnsupportedKind));
    }
}
