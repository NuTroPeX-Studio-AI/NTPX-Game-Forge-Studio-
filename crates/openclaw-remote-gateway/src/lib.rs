//! Remote gateway planning contracts for NTPX GameForge Ω∞.
//!
//! This crate is intentionally transport-neutral and performs no network connection or credential
//! operation. Concrete remote transport remains behind the Autonomy Governor boundary.

use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteEnvelope {
    pub request_id: String,
    pub operation: String,
    pub payload_digest_sha256: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteQueuePolicy {
    pub max_queue_depth: u32,
    pub reconnect_backoff_ms: u64,
    pub heartbeat_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayContractError {
    EmptyRequestId,
    EmptyOperation,
    EmptyIdempotencyKey,
    InvalidDigest,
    InvalidQueuePolicy,
}

impl fmt::Display for GatewayContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "remote gateway contract validation failed: {self:?}")
    }
}

impl Error for GatewayContractError {}

/// Validate an offline-safe request envelope before any future governed transport adapter sees it.
///
/// # Errors
/// Returns [`GatewayContractError`] for malformed identifiers, digests, or idempotency metadata.
pub fn validate_envelope(envelope: &RemoteEnvelope) -> Result<(), GatewayContractError> {
    if envelope.request_id.trim().is_empty() {
        return Err(GatewayContractError::EmptyRequestId);
    }
    if envelope.operation.trim().is_empty() {
        return Err(GatewayContractError::EmptyOperation);
    }
    if envelope.idempotency_key.trim().is_empty() {
        return Err(GatewayContractError::EmptyIdempotencyKey);
    }
    if envelope.payload_digest_sha256.len() != 64
        || !envelope
            .payload_digest_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(GatewayContractError::InvalidDigest);
    }
    Ok(())
}

impl RemoteQueuePolicy {
    /// Validate bounded queue and reconnect/liveness settings.
    ///
    /// # Errors
    /// Returns [`GatewayContractError::InvalidQueuePolicy`] if any bound is zero.
    pub fn validate(&self) -> Result<(), GatewayContractError> {
        if self.max_queue_depth == 0
            || self.reconnect_backoff_ms == 0
            || self.heartbeat_interval_ms == 0
        {
            return Err(GatewayContractError::InvalidQueuePolicy);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_digest_fails_closed() {
        let envelope = RemoteEnvelope {
            request_id: "r1".to_owned(),
            operation: "status".to_owned(),
            payload_digest_sha256: "bad".to_owned(),
            idempotency_key: "once-1".to_owned(),
        };
        assert_eq!(
            validate_envelope(&envelope),
            Err(GatewayContractError::InvalidDigest)
        );
    }
}
