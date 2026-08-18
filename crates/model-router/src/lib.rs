//! Provider-neutral deterministic model routing for NTPX GameForge Ω∞.
//!
//! The router selects among declared model capabilities and constraints. It does not perform
//! network calls, hold credentials, invoke a model, or grant execution authority. Any later
//! adapter that performs governed work must consume the verified Autonomy Governor boundary.

use std::{
    cmp::Ordering,
    collections::BTreeSet,
    error::Error,
    fmt,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Stable capability vocabulary used by model descriptors and routing requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    Text,
    Code,
    Reasoning,
    Vision,
    Audio,
    Speech,
    Embeddings,
    StructuredOutput,
    ToolUse,
}

/// Execution locality declared by a provider adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderClass {
    Local,
    Cloud,
}

/// Stable provider/model identity used in routing evidence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ModelIdentity {
    pub provider_id: String,
    pub model_id: String,
}

/// Provider-neutral model metadata used for filtering, ranking, and evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub provider_id: String,
    pub model_id: String,
    pub provider_class: ProviderClass,
    pub capabilities: BTreeSet<ModelCapability>,
    pub context_window_tokens: u64,
    pub max_output_tokens: u64,
    pub expected_latency_ms: u64,
    pub input_cost_microusd_per_million_tokens: u64,
    pub output_cost_microusd_per_million_tokens: u64,
    pub reliability_basis_points: u16,
}

impl ModelDescriptor {
    /// Return the stable identity carried by this descriptor.
    #[must_use]
    pub fn identity(&self) -> ModelIdentity {
        ModelIdentity {
            provider_id: self.provider_id.clone(),
            model_id: self.model_id.clone(),
        }
    }
}

/// Read-only catalog contract for interchangeable provider adapters.
///
/// This trait intentionally exposes no invocation, credential, filesystem, repository, or network
/// method. Phase 04 adapters describe selectable models only.
pub trait ProviderAdapter: Send + Sync {
    fn provider_id(&self) -> &str;
    fn provider_class(&self) -> ProviderClass;
    fn models(&self) -> &[ModelDescriptor];
}

/// Adapter-construction failure for inconsistent catalog metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    ProviderMismatch {
        adapter_provider_id: String,
        descriptor_provider_id: String,
    },
    ClassMismatch {
        provider_id: String,
        expected: ProviderClass,
        actual: ProviderClass,
    },
    InvalidReliability {
        identity: ModelIdentity,
        reliability_basis_points: u16,
    },
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderMismatch {
                adapter_provider_id,
                descriptor_provider_id,
            } => write!(
                formatter,
                "adapter provider '{adapter_provider_id}' cannot expose descriptor provider '{descriptor_provider_id}'"
            ),
            Self::ClassMismatch {
                provider_id,
                expected,
                actual,
            } => write!(
                formatter,
                "provider '{provider_id}' descriptor class {actual:?} does not match adapter class {expected:?}"
            ),
            Self::InvalidReliability {
                identity,
                reliability_basis_points,
            } => write!(
                formatter,
                "model '{}/{}' has invalid reliability basis points {reliability_basis_points}",
                identity.provider_id, identity.model_id
            ),
        }
    }
}

impl Error for AdapterError {}

/// Offline/local catalog adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalProviderAdapter {
    provider_id: String,
    models: Vec<ModelDescriptor>,
}

impl LocalProviderAdapter {
    /// Create a local catalog adapter after validating descriptor ownership/classification.
    ///
    /// # Errors
    /// Returns [`AdapterError`] when any descriptor belongs to a different provider, declares a
    /// cloud class, or carries reliability above 10,000 basis points.
    pub fn new(
        provider_id: impl Into<String>,
        models: Vec<ModelDescriptor>,
    ) -> Result<Self, AdapterError> {
        let provider_id = provider_id.into();
        validate_adapter_models(&provider_id, ProviderClass::Local, &models)?;
        Ok(Self {
            provider_id,
            models,
        })
    }
}

impl ProviderAdapter for LocalProviderAdapter {
    fn provider_id(&self) -> &str {
        &self.provider_id
    }

    fn provider_class(&self) -> ProviderClass {
        ProviderClass::Local
    }

    fn models(&self) -> &[ModelDescriptor] {
        &self.models
    }
}

/// Cloud catalog adapter. This type describes remote models but performs no remote operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudProviderAdapter {
    provider_id: String,
    models: Vec<ModelDescriptor>,
}

impl CloudProviderAdapter {
    /// Create a cloud catalog adapter after validating descriptor ownership/classification.
    ///
    /// # Errors
    /// Returns [`AdapterError`] when any descriptor belongs to a different provider, declares a
    /// local class, or carries reliability above 10,000 basis points.
    pub fn new(
        provider_id: impl Into<String>,
        models: Vec<ModelDescriptor>,
    ) -> Result<Self, AdapterError> {
        let provider_id = provider_id.into();
        validate_adapter_models(&provider_id, ProviderClass::Cloud, &models)?;
        Ok(Self {
            provider_id,
            models,
        })
    }
}

impl ProviderAdapter for CloudProviderAdapter {
    fn provider_id(&self) -> &str {
        &self.provider_id
    }

    fn provider_class(&self) -> ProviderClass {
        ProviderClass::Cloud
    }

    fn models(&self) -> &[ModelDescriptor] {
        &self.models
    }
}

/// Privacy boundary for one routing request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyConstraint {
    LocalOnly,
    CloudAllowed,
}

/// Model-selection requirements. Payload content is deliberately absent from this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingRequest {
    pub request_id: String,
    pub required_capabilities: BTreeSet<ModelCapability>,
    pub required_context_tokens: u64,
    pub required_output_tokens: u64,
    pub expected_input_tokens: u64,
    pub expected_output_tokens: u64,
    pub privacy: PrivacyConstraint,
    pub max_estimated_cost_microusd: Option<u64>,
    pub max_expected_latency_ms: Option<u64>,
    pub min_reliability_basis_points: Option<u16>,
}

/// Ordered ranking dimension. Earlier entries in a policy have greater precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingDimension {
    Reliability,
    Latency,
    Cost,
    Locality,
}

/// Deterministic lexicographic ranking policy and fallback bound.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingPolicy {
    pub priorities: Vec<RoutingDimension>,
    pub max_fallbacks: usize,
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        Self {
            priorities: vec![
                RoutingDimension::Reliability,
                RoutingDimension::Latency,
                RoutingDimension::Cost,
                RoutingDimension::Locality,
            ],
            max_fallbacks: 3,
        }
    }
}

/// One eligible model with the request-specific cost estimate used by routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCandidate {
    pub identity: ModelIdentity,
    pub provider_class: ProviderClass,
    pub estimated_cost_microusd: u64,
    pub expected_latency_ms: u64,
    pub reliability_basis_points: u16,
    pub context_window_tokens: u64,
    pub max_output_tokens: u64,
}

/// Reason a model was excluded before ranking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum RejectionReason {
    MissingCapability { capability: ModelCapability },
    ContextWindowTooSmall { required: u64, available: u64 },
    OutputLimitTooSmall { required: u64, available: u64 },
    PrivacyRequiresLocal,
    CostLimitExceeded { limit: u64, estimated: u64 },
    LatencyLimitExceeded { limit: u64, expected: u64 },
    ReliabilityBelowMinimum { minimum: u16, actual: u16 },
}

/// Excluded model and every deterministic exclusion reason found for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedCandidate {
    pub identity: ModelIdentity,
    pub reasons: Vec<RejectionReason>,
}

/// Final selection state. Abstention is a first-class safe outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum RoutingOutcome {
    Selected {
        primary: ModelCandidate,
        fallbacks: Vec<ModelCandidate>,
    },
    Abstained {
        reason: String,
    },
}

/// Complete deterministic routing evidence, excluding its own digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingEvidence {
    pub request: RoutingRequest,
    pub policy: RoutingPolicy,
    pub eligible: Vec<ModelCandidate>,
    pub rejected: Vec<RejectedCandidate>,
    pub outcome: RoutingOutcome,
}

/// Routing evidence plus a SHA-256 digest of its canonical serialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub evidence_sha256: String,
    pub evidence: RoutingEvidence,
}

/// Model-router failures that prevent a trustworthy decision from being emitted.
#[derive(Debug)]
pub enum RouterError {
    InvalidPolicy(String),
    AdapterContractViolation(String),
    DuplicateModelIdentity(ModelIdentity),
    InvalidReliability {
        identity: ModelIdentity,
        reliability_basis_points: u16,
    },
    Encode(serde_json::Error),
}

impl fmt::Display for RouterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy(reason) => write!(formatter, "invalid routing policy: {reason}"),
            Self::AdapterContractViolation(reason) => {
                write!(formatter, "provider adapter contract violation: {reason}")
            }
            Self::DuplicateModelIdentity(identity) => write!(
                formatter,
                "duplicate model identity '{}/{}'",
                identity.provider_id, identity.model_id
            ),
            Self::InvalidReliability {
                identity,
                reliability_basis_points,
            } => write!(
                formatter,
                "model '{}/{}' has invalid reliability basis points {reliability_basis_points}",
                identity.provider_id, identity.model_id
            ),
            Self::Encode(error) => write!(formatter, "routing evidence encoding failed: {error}"),
        }
    }
}

impl Error for RouterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::InvalidPolicy(_)
            | Self::AdapterContractViolation(_)
            | Self::DuplicateModelIdentity(_)
            | Self::InvalidReliability { .. } => None,
        }
    }
}

/// Stateless deterministic provider-neutral router.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModelRouter;

impl ModelRouter {
    /// Filter and rank provider catalogs without invoking any model.
    ///
    /// Catalog order and adapter enumeration order do not affect the final result. Identity is the
    /// final tie-breaker. The returned fallback list is already ordered for caller-controlled retry.
    ///
    /// # Errors
    /// Returns [`RouterError`] for invalid policy, duplicate model identity, malformed model
    /// reliability, adapter metadata inconsistency, or evidence-serialization failure.
    pub fn route(
        adapters: &[&dyn ProviderAdapter],
        request: &RoutingRequest,
        policy: &RoutingPolicy,
    ) -> Result<RoutingDecision, RouterError> {
        validate_policy(policy)?;
        let mut seen = BTreeSet::new();
        let mut descriptors = Vec::new();

        for adapter in adapters {
            for descriptor in adapter.models() {
                validate_runtime_descriptor(*adapter, descriptor)?;
                let identity = descriptor.identity();
                if !seen.insert(identity.clone()) {
                    return Err(RouterError::DuplicateModelIdentity(identity));
                }
                descriptors.push(descriptor.clone());
            }
        }

        descriptors.sort_by_key(ModelDescriptor::identity);

        let mut eligible = Vec::new();
        let mut rejected = Vec::new();
        for descriptor in &descriptors {
            let candidate = candidate_for(descriptor, request);
            let reasons = rejection_reasons(descriptor, &candidate, request);
            if reasons.is_empty() {
                eligible.push(candidate);
            } else {
                rejected.push(RejectedCandidate {
                    identity: descriptor.identity(),
                    reasons,
                });
            }
        }

        eligible.sort_by(|left, right| compare_candidates(left, right, policy));
        rejected.sort_by(|left, right| left.identity.cmp(&right.identity));

        let outcome = match eligible.first() {
            Some(primary) => RoutingOutcome::Selected {
                primary: primary.clone(),
                fallbacks: eligible
                    .iter()
                    .skip(1)
                    .take(policy.max_fallbacks)
                    .cloned()
                    .collect(),
            },
            None => RoutingOutcome::Abstained {
                reason: "no model satisfies all routing constraints".to_owned(),
            },
        };

        let evidence = RoutingEvidence {
            request: request.clone(),
            policy: policy.clone(),
            eligible,
            rejected,
            outcome,
        };
        let evidence_sha256 = evidence_digest(&evidence)?;
        Ok(RoutingDecision {
            evidence_sha256,
            evidence,
        })
    }
}

fn validate_adapter_models(
    provider_id: &str,
    provider_class: ProviderClass,
    models: &[ModelDescriptor],
) -> Result<(), AdapterError> {
    for model in models {
        if model.provider_id != provider_id {
            return Err(AdapterError::ProviderMismatch {
                adapter_provider_id: provider_id.to_owned(),
                descriptor_provider_id: model.provider_id.clone(),
            });
        }
        if model.provider_class != provider_class {
            return Err(AdapterError::ClassMismatch {
                provider_id: provider_id.to_owned(),
                expected: provider_class,
                actual: model.provider_class,
            });
        }
        if model.reliability_basis_points > 10_000 {
            return Err(AdapterError::InvalidReliability {
                identity: model.identity(),
                reliability_basis_points: model.reliability_basis_points,
            });
        }
    }
    Ok(())
}

fn validate_runtime_descriptor(
    adapter: &dyn ProviderAdapter,
    descriptor: &ModelDescriptor,
) -> Result<(), RouterError> {
    if descriptor.provider_id != adapter.provider_id() {
        return Err(RouterError::AdapterContractViolation(format!(
            "adapter '{}' exposed descriptor for '{}'",
            adapter.provider_id(), descriptor.provider_id
        )));
    }
    if descriptor.provider_class != adapter.provider_class() {
        return Err(RouterError::AdapterContractViolation(format!(
            "adapter '{}' declared {:?} but descriptor '{}' declared {:?}",
            adapter.provider_id(),
            adapter.provider_class(),
            descriptor.model_id,
            descriptor.provider_class
        )));
    }
    if descriptor.reliability_basis_points > 10_000 {
        return Err(RouterError::InvalidReliability {
            identity: descriptor.identity(),
            reliability_basis_points: descriptor.reliability_basis_points,
        });
    }
    Ok(())
}

fn validate_policy(policy: &RoutingPolicy) -> Result<(), RouterError> {
    if policy.priorities.is_empty() {
        return Err(RouterError::InvalidPolicy(
            "at least one ranking dimension is required".to_owned(),
        ));
    }
    let unique = policy.priorities.iter().copied().collect::<BTreeSet<_>>();
    if unique.len() != policy.priorities.len() {
        return Err(RouterError::InvalidPolicy(
            "ranking dimensions must not repeat".to_owned(),
        ));
    }
    Ok(())
}

fn candidate_for(descriptor: &ModelDescriptor, request: &RoutingRequest) -> ModelCandidate {
    ModelCandidate {
        identity: descriptor.identity(),
        provider_class: descriptor.provider_class,
        estimated_cost_microusd: estimate_cost_microusd(descriptor, request),
        expected_latency_ms: descriptor.expected_latency_ms,
        reliability_basis_points: descriptor.reliability_basis_points,
        context_window_tokens: descriptor.context_window_tokens,
        max_output_tokens: descriptor.max_output_tokens,
    }
}

fn estimate_cost_microusd(descriptor: &ModelDescriptor, request: &RoutingRequest) -> u64 {
    let input = descriptor
        .input_cost_microusd_per_million_tokens
        .saturating_mul(request.expected_input_tokens)
        .div_ceil(1_000_000);
    let output = descriptor
        .output_cost_microusd_per_million_tokens
        .saturating_mul(request.expected_output_tokens)
        .div_ceil(1_000_000);
    input.saturating_add(output)
}

fn rejection_reasons(
    descriptor: &ModelDescriptor,
    candidate: &ModelCandidate,
    request: &RoutingRequest,
) -> Vec<RejectionReason> {
    let mut reasons = Vec::new();
    for capability in &request.required_capabilities {
        if !descriptor.capabilities.contains(capability) {
            reasons.push(RejectionReason::MissingCapability {
                capability: *capability,
            });
        }
    }
    if descriptor.context_window_tokens < request.required_context_tokens {
        reasons.push(RejectionReason::ContextWindowTooSmall {
            required: request.required_context_tokens,
            available: descriptor.context_window_tokens,
        });
    }
    if descriptor.max_output_tokens < request.required_output_tokens {
        reasons.push(RejectionReason::OutputLimitTooSmall {
            required: request.required_output_tokens,
            available: descriptor.max_output_tokens,
        });
    }
    if request.privacy == PrivacyConstraint::LocalOnly
        && descriptor.provider_class != ProviderClass::Local
    {
        reasons.push(RejectionReason::PrivacyRequiresLocal);
    }
    if let Some(limit) = request.max_estimated_cost_microusd {
        if candidate.estimated_cost_microusd > limit {
            reasons.push(RejectionReason::CostLimitExceeded {
                limit,
                estimated: candidate.estimated_cost_microusd,
            });
        }
    }
    if let Some(limit) = request.max_expected_latency_ms {
        if descriptor.expected_latency_ms > limit {
            reasons.push(RejectionReason::LatencyLimitExceeded {
                limit,
                expected: descriptor.expected_latency_ms,
            });
        }
    }
    if let Some(minimum) = request.min_reliability_basis_points {
        if descriptor.reliability_basis_points < minimum {
            reasons.push(RejectionReason::ReliabilityBelowMinimum {
                minimum,
                actual: descriptor.reliability_basis_points,
            });
        }
    }
    reasons
}

fn compare_candidates(
    left: &ModelCandidate,
    right: &ModelCandidate,
    policy: &RoutingPolicy,
) -> Ordering {
    for dimension in &policy.priorities {
        let ordering = match dimension {
            RoutingDimension::Reliability => right
                .reliability_basis_points
                .cmp(&left.reliability_basis_points),
            RoutingDimension::Latency => left.expected_latency_ms.cmp(&right.expected_latency_ms),
            RoutingDimension::Cost => left
                .estimated_cost_microusd
                .cmp(&right.estimated_cost_microusd),
            RoutingDimension::Locality => left.provider_class.cmp(&right.provider_class),
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.identity.cmp(&right.identity)
}

fn evidence_digest(evidence: &RoutingEvidence) -> Result<String, RouterError> {
    let encoded = serde_json::to_vec(evidence).map_err(RouterError::Encode)?;
    let digest = Sha256::digest(encoded);
    Ok(to_lower_hex(&digest))
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
