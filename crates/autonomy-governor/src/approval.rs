use std::{error::Error, fmt};

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::Request;

type HmacSha256 = Hmac<Sha256>;
const MINIMUM_KEY_BYTES: usize = 32;

/// Human approval proof bound to one exact authorization request and one nonce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalProof {
    pub authority: String,
    pub request_digest: String,
    pub nonce: String,
    pub issued_at_unix: u64,
    pub expires_at_unix: u64,
    pub tag: String,
}

/// Verified approval evidence safe to persist in the audit record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalEvidence {
    pub authority: String,
    pub request_digest: String,
    pub nonce: String,
    pub issued_at_unix: u64,
    pub expires_at_unix: u64,
}

#[derive(Serialize)]
struct ApprovalPayload<'a> {
    authority: &'a str,
    request_digest: &'a str,
    nonce: &'a str,
    issued_at_unix: u64,
    expires_at_unix: u64,
}

/// Approval proof creation or verification failure.
#[derive(Debug)]
pub enum ApprovalError {
    WeakKey { actual: usize },
    EmptyAuthority,
    EmptyNonce,
    InvalidLifetime,
    NotYetValid,
    Expired,
    RequestDigestMismatch,
    InvalidTagEncoding,
    InvalidTag,
    Encode(serde_json::Error),
}

impl fmt::Display for ApprovalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WeakKey { actual } => write!(
                formatter,
                "approval key is too short: {actual} bytes; minimum is {MINIMUM_KEY_BYTES}"
            ),
            Self::EmptyAuthority => write!(formatter, "approval authority must not be empty"),
            Self::EmptyNonce => write!(formatter, "approval nonce must not be empty"),
            Self::InvalidLifetime => write!(
                formatter,
                "approval expiry must be strictly later than its issue time"
            ),
            Self::NotYetValid => write!(formatter, "approval proof is not yet valid"),
            Self::Expired => write!(formatter, "approval proof has expired"),
            Self::RequestDigestMismatch => {
                write!(formatter, "approval proof does not match the exact request")
            }
            Self::InvalidTagEncoding => write!(formatter, "approval proof tag is not valid hex"),
            Self::InvalidTag => write!(formatter, "approval proof authentication failed"),
            Self::Encode(error) => write!(formatter, "approval proof encoding failed: {error}"),
        }
    }
}

impl Error for ApprovalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            _ => None,
        }
    }
}

/// HMAC approval signer. The key is never exposed through `Debug` or serialization.
pub struct ApprovalSigner {
    key: Vec<u8>,
}

impl fmt::Debug for ApprovalSigner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApprovalSigner")
            .field("key", &"[REDACTED]")
            .finish()
    }
}

impl ApprovalSigner {
    /// Construct a signer from an externally supplied human-approval secret.
    ///
    /// # Errors
    /// Returns [`ApprovalError::WeakKey`] when fewer than 32 bytes are supplied.
    pub fn new(key: impl AsRef<[u8]>) -> Result<Self, ApprovalError> {
        let key = key.as_ref();
        validate_key(key)?;
        Ok(Self { key: key.to_vec() })
    }

    /// Issue one request-bound approval proof.
    ///
    /// This helper does not decide whether a human actually approved the request; callers must
    /// invoke it only from an explicitly authorized human approval boundary.
    ///
    /// # Errors
    /// Returns [`ApprovalError`] for invalid metadata or serialization failures.
    pub fn issue(
        &self,
        request: &Request,
        authority: impl Into<String>,
        nonce: impl Into<String>,
        issued_at_unix: u64,
        expires_at_unix: u64,
    ) -> Result<ApprovalProof, ApprovalError> {
        let authority = authority.into();
        let nonce = nonce.into();
        validate_metadata(&authority, &nonce, issued_at_unix, expires_at_unix)?;
        let request_digest = request_digest(request)?;
        let tag = compute_tag(
            &self.key,
            &authority,
            &request_digest,
            &nonce,
            issued_at_unix,
            expires_at_unix,
        )?;
        Ok(ApprovalProof {
            authority,
            request_digest,
            nonce,
            issued_at_unix,
            expires_at_unix,
            tag,
        })
    }
}

/// HMAC approval verifier. The key is never exposed through `Debug` or serialization.
pub struct ApprovalVerifier {
    key: Vec<u8>,
}

impl fmt::Debug for ApprovalVerifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApprovalVerifier")
            .field("key", &"[REDACTED]")
            .finish()
    }
}

impl ApprovalVerifier {
    /// Construct a verifier from the shared approval verification secret.
    ///
    /// # Errors
    /// Returns [`ApprovalError::WeakKey`] when fewer than 32 bytes are supplied.
    pub fn new(key: impl AsRef<[u8]>) -> Result<Self, ApprovalError> {
        let key = key.as_ref();
        validate_key(key)?;
        Ok(Self { key: key.to_vec() })
    }

    /// Verify authenticity, request binding, lifetime, and metadata for one approval proof.
    ///
    /// Replay prevention is enforced by the governor's audited approval path, which records the
    /// verified nonce before any governed external capability may run.
    ///
    /// # Errors
    /// Returns [`ApprovalError`] when the proof is malformed, expired, forged, or bound to a
    /// different request.
    pub fn verify(
        &self,
        request: &Request,
        proof: &ApprovalProof,
        now_unix: u64,
    ) -> Result<ApprovalEvidence, ApprovalError> {
        validate_metadata(
            &proof.authority,
            &proof.nonce,
            proof.issued_at_unix,
            proof.expires_at_unix,
        )?;
        if now_unix < proof.issued_at_unix {
            return Err(ApprovalError::NotYetValid);
        }
        if now_unix >= proof.expires_at_unix {
            return Err(ApprovalError::Expired);
        }

        let expected_digest = request_digest(request)?;
        if proof.request_digest != expected_digest {
            return Err(ApprovalError::RequestDigestMismatch);
        }

        let decoded_tag = decode_hex(&proof.tag).ok_or(ApprovalError::InvalidTagEncoding)?;
        let payload = payload_bytes(
            &proof.authority,
            &proof.request_digest,
            &proof.nonce,
            proof.issued_at_unix,
            proof.expires_at_unix,
        )?;
        let mut mac = HmacSha256::new_from_slice(&self.key)
            .expect("validated HMAC-SHA256 keys accept arbitrary non-empty lengths");
        mac.update(&payload);
        mac.verify_slice(&decoded_tag)
            .map_err(|_| ApprovalError::InvalidTag)?;

        Ok(ApprovalEvidence {
            authority: proof.authority.clone(),
            request_digest: proof.request_digest.clone(),
            nonce: proof.nonce.clone(),
            issued_at_unix: proof.issued_at_unix,
            expires_at_unix: proof.expires_at_unix,
        })
    }
}

fn validate_key(key: &[u8]) -> Result<(), ApprovalError> {
    if key.len() < MINIMUM_KEY_BYTES {
        Err(ApprovalError::WeakKey { actual: key.len() })
    } else {
        Ok(())
    }
}

fn validate_metadata(
    authority: &str,
    nonce: &str,
    issued_at_unix: u64,
    expires_at_unix: u64,
) -> Result<(), ApprovalError> {
    if authority.trim().is_empty() {
        return Err(ApprovalError::EmptyAuthority);
    }
    if nonce.trim().is_empty() {
        return Err(ApprovalError::EmptyNonce);
    }
    if expires_at_unix <= issued_at_unix {
        return Err(ApprovalError::InvalidLifetime);
    }
    Ok(())
}

fn request_digest(request: &Request) -> Result<String, ApprovalError> {
    let encoded = serde_json::to_vec(request).map_err(ApprovalError::Encode)?;
    Ok(to_lower_hex(&Sha256::digest(encoded)))
}

fn compute_tag(
    key: &[u8],
    authority: &str,
    request_digest: &str,
    nonce: &str,
    issued_at_unix: u64,
    expires_at_unix: u64,
) -> Result<String, ApprovalError> {
    let payload = payload_bytes(
        authority,
        request_digest,
        nonce,
        issued_at_unix,
        expires_at_unix,
    )?;
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("validated HMAC-SHA256 keys accept arbitrary non-empty lengths");
    mac.update(&payload);
    Ok(to_lower_hex(&mac.finalize().into_bytes()))
}

fn payload_bytes(
    authority: &str,
    request_digest: &str,
    nonce: &str,
    issued_at_unix: u64,
    expires_at_unix: u64,
) -> Result<Vec<u8>, ApprovalError> {
    serde_json::to_vec(&ApprovalPayload {
        authority,
        request_digest,
        nonce,
        issued_at_unix,
        expires_at_unix,
    })
    .map_err(ApprovalError::Encode)
}

fn decode_hex(input: &str) -> Option<Vec<u8>> {
    if !input.len().is_multiple_of(2) {
        return None;
    }
    input
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0])?;
            let low = hex_nibble(pair[1])?;
            Some((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
