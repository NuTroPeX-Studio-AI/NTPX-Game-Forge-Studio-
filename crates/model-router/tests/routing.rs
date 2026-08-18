use std::collections::BTreeSet;

use model_router::{
    AdapterError, CloudProviderAdapter, LocalProviderAdapter, ModelCapability, ModelDescriptor,
    ModelRouter, PrivacyConstraint, ProviderAdapter, ProviderClass, RejectionReason,
    RouterError, RoutingDimension, RoutingOutcome, RoutingPolicy, RoutingRequest,
};

fn capabilities(values: &[ModelCapability]) -> BTreeSet<ModelCapability> {
    values.iter().copied().collect()
}

fn descriptor(
    provider_id: &str,
    model_id: &str,
    provider_class: ProviderClass,
    reliability_basis_points: u16,
    latency_ms: u64,
    input_cost: u64,
    output_cost: u64,
) -> ModelDescriptor {
    ModelDescriptor {
        provider_id: provider_id.to_owned(),
        model_id: model_id.to_owned(),
        provider_class,
        capabilities: capabilities(&[
            ModelCapability::Text,
            ModelCapability::Code,
            ModelCapability::Reasoning,
            ModelCapability::StructuredOutput,
        ]),
        context_window_tokens: 128_000,
        max_output_tokens: 16_000,
        expected_latency_ms: latency_ms,
        input_cost_microusd_per_million_tokens: input_cost,
        output_cost_microusd_per_million_tokens: output_cost,
        reliability_basis_points,
    }
}

fn request(privacy: PrivacyConstraint) -> RoutingRequest {
    RoutingRequest {
        request_id: "route-001".to_owned(),
        required_capabilities: capabilities(&[
            ModelCapability::Code,
            ModelCapability::Reasoning,
            ModelCapability::StructuredOutput,
        ]),
        required_context_tokens: 32_000,
        required_output_tokens: 4_000,
        expected_input_tokens: 12_000,
        expected_output_tokens: 2_000,
        privacy,
        max_estimated_cost_microusd: None,
        max_expected_latency_ms: None,
        min_reliability_basis_points: None,
    }
}

fn adapters() -> (LocalProviderAdapter, CloudProviderAdapter) {
    let local = LocalProviderAdapter::new(
        "local-runtime",
        vec![descriptor(
            "local-runtime",
            "code-local",
            ProviderClass::Local,
            9_500,
            900,
            0,
            0,
        )],
    )
    .expect("valid local adapter");
    let cloud = CloudProviderAdapter::new(
        "cloud-catalog",
        vec![
            descriptor(
                "cloud-catalog",
                "code-premium",
                ProviderClass::Cloud,
                9_800,
                500,
                800,
                2_000,
            ),
            descriptor(
                "cloud-catalog",
                "code-fast",
                ProviderClass::Cloud,
                9_600,
                250,
                300,
                800,
            ),
        ],
    )
    .expect("valid cloud adapter");
    (local, cloud)
}

#[test]
fn adapter_contracts_reject_wrong_provider_and_class() {
    let wrong_provider = descriptor(
        "other",
        "model",
        ProviderClass::Local,
        9_000,
        100,
        0,
        0,
    );
    assert!(matches!(
        LocalProviderAdapter::new("local", vec![wrong_provider]),
        Err(AdapterError::ProviderMismatch { .. })
    ));

    let wrong_class = descriptor(
        "local",
        "model",
        ProviderClass::Cloud,
        9_000,
        100,
        0,
        0,
    );
    assert!(matches!(
        LocalProviderAdapter::new("local", vec![wrong_class]),
        Err(AdapterError::ClassMismatch { .. })
    ));
}

#[test]
fn local_only_privacy_excludes_cloud_candidates() {
    let (local, cloud) = adapters();
    let decision = ModelRouter::route(
        &[&cloud as &dyn ProviderAdapter, &local],
        &request(PrivacyConstraint::LocalOnly),
        &RoutingPolicy::default(),
    )
    .expect("route should succeed");

    match &decision.evidence.outcome {
        RoutingOutcome::Selected { primary, fallbacks } => {
            assert_eq!(primary.identity.provider_id, "local-runtime");
            assert!(fallbacks.is_empty());
        }
        RoutingOutcome::Abstained { .. } => panic!("local model should be eligible"),
    }
    assert_eq!(decision.evidence.rejected.len(), 2);
    assert!(decision.evidence.rejected.iter().all(|rejected| rejected
        .reasons
        .contains(&RejectionReason::PrivacyRequiresLocal)));
}

#[test]
fn default_policy_prefers_reliability_and_emits_ordered_fallbacks() {
    let (local, cloud) = adapters();
    let decision = ModelRouter::route(
        &[&local, &cloud],
        &request(PrivacyConstraint::CloudAllowed),
        &RoutingPolicy::default(),
    )
    .expect("route should succeed");

    match &decision.evidence.outcome {
        RoutingOutcome::Selected { primary, fallbacks } => {
            assert_eq!(primary.identity.model_id, "code-premium");
            assert_eq!(fallbacks.len(), 2);
            assert_eq!(fallbacks[0].identity.model_id, "code-fast");
            assert_eq!(fallbacks[1].identity.model_id, "code-local");
        }
        RoutingOutcome::Abstained { .. } => panic!("models should be eligible"),
    }
}

#[test]
fn locality_first_policy_can_prefer_local_without_changing_eligibility() {
    let (local, cloud) = adapters();
    let policy = RoutingPolicy {
        priorities: vec![
            RoutingDimension::Locality,
            RoutingDimension::Reliability,
            RoutingDimension::Latency,
            RoutingDimension::Cost,
        ],
        max_fallbacks: 2,
    };
    let decision = ModelRouter::route(
        &[&cloud, &local],
        &request(PrivacyConstraint::CloudAllowed),
        &policy,
    )
    .expect("route should succeed");

    match decision.evidence.outcome {
        RoutingOutcome::Selected { primary, .. } => {
            assert_eq!(primary.identity.provider_id, "local-runtime");
        }
        RoutingOutcome::Abstained { .. } => panic!("models should be eligible"),
    }
}

#[test]
fn cost_latency_and_reliability_constraints_fail_closed_to_abstention() {
    let (local, cloud) = adapters();
    let mut constrained = request(PrivacyConstraint::CloudAllowed);
    constrained.max_estimated_cost_microusd = Some(1);
    constrained.max_expected_latency_ms = Some(100);
    constrained.min_reliability_basis_points = Some(9_900);

    let decision = ModelRouter::route(
        &[&local, &cloud],
        &constrained,
        &RoutingPolicy::default(),
    )
    .expect("constraint evaluation should succeed");

    assert!(matches!(
        decision.evidence.outcome,
        RoutingOutcome::Abstained { .. }
    ));
    assert!(decision.evidence.eligible.is_empty());
    assert_eq!(decision.evidence.rejected.len(), 3);
}

#[test]
fn missing_capability_and_capacity_are_recorded_as_evidence() {
    let limited = ModelDescriptor {
        provider_id: "local-runtime".to_owned(),
        model_id: "limited".to_owned(),
        provider_class: ProviderClass::Local,
        capabilities: capabilities(&[ModelCapability::Text]),
        context_window_tokens: 8_000,
        max_output_tokens: 512,
        expected_latency_ms: 10,
        input_cost_microusd_per_million_tokens: 0,
        output_cost_microusd_per_million_tokens: 0,
        reliability_basis_points: 10_000,
    };
    let local = LocalProviderAdapter::new("local-runtime", vec![limited])
        .expect("valid local adapter");
    let decision = ModelRouter::route(
        &[&local],
        &request(PrivacyConstraint::LocalOnly),
        &RoutingPolicy::default(),
    )
    .expect("route should produce abstention evidence");

    let reasons = &decision.evidence.rejected[0].reasons;
    assert!(reasons.iter().any(|reason| matches!(
        reason,
        RejectionReason::MissingCapability {
            capability: ModelCapability::Code
        }
    )));
    assert!(reasons
        .iter()
        .any(|reason| matches!(reason, RejectionReason::ContextWindowTooSmall { .. })));
    assert!(reasons
        .iter()
        .any(|reason| matches!(reason, RejectionReason::OutputLimitTooSmall { .. })));
}

#[test]
fn decision_and_digest_are_independent_of_adapter_enumeration_order() {
    let (local, cloud) = adapters();
    let request = request(PrivacyConstraint::CloudAllowed);
    let policy = RoutingPolicy::default();

    let left = ModelRouter::route(&[&local, &cloud], &request, &policy).expect("left route");
    let right = ModelRouter::route(&[&cloud, &local], &request, &policy).expect("right route");

    assert_eq!(left, right);
    assert_eq!(left.evidence_sha256.len(), 64);
    assert_eq!(
        serde_json::to_string(&left).expect("serialize left"),
        serde_json::to_string(&right).expect("serialize right")
    );
}

#[test]
fn duplicate_identity_and_invalid_policy_are_rejected() {
    let duplicate = descriptor(
        "same",
        "model",
        ProviderClass::Cloud,
        9_000,
        100,
        0,
        0,
    );
    let first = CloudProviderAdapter::new("same", vec![duplicate.clone()]).expect("first");
    let second = CloudProviderAdapter::new("same", vec![duplicate]).expect("second");

    assert!(matches!(
        ModelRouter::route(
            &[&first, &second],
            &request(PrivacyConstraint::CloudAllowed),
            &RoutingPolicy::default()
        ),
        Err(RouterError::DuplicateModelIdentity(_))
    ));

    let invalid = RoutingPolicy {
        priorities: vec![RoutingDimension::Cost, RoutingDimension::Cost],
        max_fallbacks: 1,
    };
    assert!(matches!(
        ModelRouter::route(
            &[&first],
            &request(PrivacyConstraint::CloudAllowed),
            &invalid
        ),
        Err(RouterError::InvalidPolicy(_))
    ));
}
