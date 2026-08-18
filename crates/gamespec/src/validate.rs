use crate::{DependencyGraph, GameSpec, SpecId, CURRENT_SCHEMA_VERSION};
use std::collections::BTreeSet;

/// Structured semantic validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub code: String,
    pub path: String,
    pub message: String,
}

/// Full semantic validation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    errors: Vec<ValidationError>,
}

impl ValidationReport {
    /// Return true when validation produced no finding.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Borrow validation findings in deterministic discovery order.
    #[must_use]
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    pub(crate) fn into_errors(self) -> Vec<ValidationError> {
        self.errors
    }
}

impl GameSpec {
    /// Produce a semantic validation report without panicking.
    #[must_use]
    pub fn validation_report(&self) -> ValidationReport {
        let mut errors = Vec::new();
        validate_version(self, &mut errors);
        let sets = collect_ids(self, &mut errors);
        validate_architecture(self, &sets.modules, &mut errors);
        validate_features(self, &sets, &mut errors);
        validate_components(self, &sets.components, &mut errors);
        validate_abilities(self, &sets.modules, &mut errors);
        validate_npcs(self, &sets.components, &mut errors);
        validate_dialogues(self, &sets.npcs, &mut errors);
        validate_quests(self, &sets.all, &mut errors);
        validate_tests(self, &sets.all, &mut errors);
        validate_cycles(self, &mut errors);
        ValidationReport { errors }
    }
}

#[derive(Default)]
struct IdSets {
    all: BTreeSet<SpecId>,
    modules: BTreeSet<SpecId>,
    components: BTreeSet<SpecId>,
    npcs: BTreeSet<SpecId>,
    tests: BTreeSet<SpecId>,
}

fn validate_version(spec: &GameSpec, errors: &mut Vec<ValidationError>) {
    if spec.schema_version != CURRENT_SCHEMA_VERSION {
        push_error(
            errors,
            "unsupported_schema_version",
            "schema_version",
            format!(
                "expected schema version {CURRENT_SCHEMA_VERSION}, found {}",
                spec.schema_version
            ),
        );
    }
}

fn collect_ids(spec: &GameSpec, errors: &mut Vec<ValidationError>) -> IdSets {
    let mut sets = IdSets::default();
    collect(&spec.project.id, "project.id", &mut sets.all, errors);
    for (index, module) in spec.architecture.modules.iter().enumerate() {
        collect(
            &module.id,
            &format!("architecture.modules[{index}].id"),
            &mut sets.all,
            errors,
        );
        sets.modules.insert(module.id.clone());
    }
    collect_slice(
        &spec.features,
        |value| &value.id,
        "features",
        &mut sets.all,
        errors,
    );
    for (index, component) in spec.components.iter().enumerate() {
        collect(
            &component.id,
            &format!("components[{index}].id"),
            &mut sets.all,
            errors,
        );
        sets.components.insert(component.id.clone());
    }
    collect_slice(
        &spec.entities,
        |value| &value.id,
        "entities",
        &mut sets.all,
        errors,
    );
    collect_slice(
        &spec.inputs,
        |value| &value.id,
        "inputs",
        &mut sets.all,
        errors,
    );
    collect_slice(
        &spec.abilities,
        |value| &value.id,
        "abilities",
        &mut sets.all,
        errors,
    );
    for (index, npc) in spec.npcs.iter().enumerate() {
        collect(&npc.id, &format!("npcs[{index}].id"), &mut sets.all, errors);
        sets.npcs.insert(npc.id.clone());
    }
    collect_slice(
        &spec.dialogues,
        |value| &value.id,
        "dialogues",
        &mut sets.all,
        errors,
    );
    collect_slice(
        &spec.quests,
        |value| &value.id,
        "quests",
        &mut sets.all,
        errors,
    );
    collect_slice(
        &spec.worlds,
        |value| &value.id,
        "worlds",
        &mut sets.all,
        errors,
    );
    collect_slice(
        &spec.assets,
        |value| &value.id,
        "assets",
        &mut sets.all,
        errors,
    );
    for (index, test) in spec.tests.iter().enumerate() {
        collect(
            &test.id,
            &format!("tests[{index}].id"),
            &mut sets.all,
            errors,
        );
        sets.tests.insert(test.id.clone());
    }
    collect_slice(
        &spec.builds,
        |value| &value.id,
        "builds",
        &mut sets.all,
        errors,
    );
    sets
}

fn collect_slice<T, F>(
    values: &[T],
    id_of: F,
    path: &str,
    all: &mut BTreeSet<SpecId>,
    errors: &mut Vec<ValidationError>,
) where
    F: Fn(&T) -> &SpecId,
{
    for (index, value) in values.iter().enumerate() {
        collect(id_of(value), &format!("{path}[{index}].id"), all, errors);
    }
}

fn collect(id: &SpecId, path: &str, all: &mut BTreeSet<SpecId>, errors: &mut Vec<ValidationError>) {
    if !id.is_valid() {
        push_error(
            errors,
            "invalid_id",
            path,
            format!("noncanonical identifier '{id}'"),
        );
    }
    if !all.insert(id.clone()) {
        push_error(
            errors,
            "duplicate_id",
            path,
            format!("duplicate identifier '{id}'"),
        );
    }
}

fn validate_architecture(
    spec: &GameSpec,
    modules: &BTreeSet<SpecId>,
    errors: &mut Vec<ValidationError>,
) {
    for (module_index, module) in spec.architecture.modules.iter().enumerate() {
        for (dependency_index, dependency) in module.depends_on.iter().enumerate() {
            require_reference(
                dependency,
                modules,
                &format!("architecture.modules[{module_index}].depends_on[{dependency_index}]"),
                errors,
            );
        }
    }
}

fn validate_features(spec: &GameSpec, sets: &IdSets, errors: &mut Vec<ValidationError>) {
    for (index, feature) in spec.features.iter().enumerate() {
        require_reference(
            &feature.owner_module,
            &sets.modules,
            &format!("features[{index}].owner_module"),
            errors,
        );
        for (dependency_index, dependency) in feature.dependencies.iter().enumerate() {
            require_reference(
                dependency,
                &sets.all,
                &format!("features[{index}].dependencies[{dependency_index}]"),
                errors,
            );
        }
        for (test_index, test) in feature.tests.iter().enumerate() {
            require_reference(
                test,
                &sets.tests,
                &format!("features[{index}].tests[{test_index}]"),
                errors,
            );
        }
    }
}

fn validate_components(
    spec: &GameSpec,
    components: &BTreeSet<SpecId>,
    errors: &mut Vec<ValidationError>,
) {
    for (entity_index, entity) in spec.entities.iter().enumerate() {
        for (component_index, component) in entity.components.iter().enumerate() {
            require_reference(
                component,
                components,
                &format!("entities[{entity_index}].components[{component_index}]"),
                errors,
            );
        }
    }
}

fn validate_abilities(
    spec: &GameSpec,
    modules: &BTreeSet<SpecId>,
    errors: &mut Vec<ValidationError>,
) {
    for (index, ability) in spec.abilities.iter().enumerate() {
        require_reference(
            &ability.owner_module,
            modules,
            &format!("abilities[{index}].owner_module"),
            errors,
        );
    }
}

fn validate_npcs(
    spec: &GameSpec,
    components: &BTreeSet<SpecId>,
    errors: &mut Vec<ValidationError>,
) {
    for (npc_index, npc) in spec.npcs.iter().enumerate() {
        for (component_index, component) in npc.components.iter().enumerate() {
            require_reference(
                component,
                components,
                &format!("npcs[{npc_index}].components[{component_index}]"),
                errors,
            );
        }
    }
}

fn validate_dialogues(spec: &GameSpec, npcs: &BTreeSet<SpecId>, errors: &mut Vec<ValidationError>) {
    for (dialogue_index, dialogue) in spec.dialogues.iter().enumerate() {
        let mut nodes = BTreeSet::new();
        for (node_index, node) in dialogue.nodes.iter().enumerate() {
            collect_local(
                &node.id,
                &format!("dialogues[{dialogue_index}].nodes[{node_index}].id"),
                &mut nodes,
                errors,
            );
        }
        for (participant_index, participant) in dialogue.participants.iter().enumerate() {
            require_reference(
                participant,
                npcs,
                &format!("dialogues[{dialogue_index}].participants[{participant_index}]"),
                errors,
            );
        }
        for (node_index, node) in dialogue.nodes.iter().enumerate() {
            require_reference(
                &node.speaker,
                npcs,
                &format!("dialogues[{dialogue_index}].nodes[{node_index}].speaker"),
                errors,
            );
            for (next_index, next) in node.next.iter().enumerate() {
                require_reference(
                    next,
                    &nodes,
                    &format!("dialogues[{dialogue_index}].nodes[{node_index}].next[{next_index}]"),
                    errors,
                );
            }
        }
    }
}

fn validate_quests(spec: &GameSpec, all: &BTreeSet<SpecId>, errors: &mut Vec<ValidationError>) {
    for (quest_index, quest) in spec.quests.iter().enumerate() {
        let mut steps = BTreeSet::new();
        for (step_index, step) in quest.steps.iter().enumerate() {
            collect_local(
                &step.id,
                &format!("quests[{quest_index}].steps[{step_index}].id"),
                &mut steps,
                errors,
            );
            for (feature_index, feature) in step.requires_features.iter().enumerate() {
                require_reference(
                    feature,
                    all,
                    &format!(
                        "quests[{quest_index}].steps[{step_index}].requires_features[{feature_index}]"
                    ),
                    errors,
                );
            }
        }
    }
}

fn validate_tests(spec: &GameSpec, all: &BTreeSet<SpecId>, errors: &mut Vec<ValidationError>) {
    for (index, test) in spec.tests.iter().enumerate() {
        require_reference(&test.target, all, &format!("tests[{index}].target"), errors);
    }
}

fn validate_cycles(spec: &GameSpec, errors: &mut Vec<ValidationError>) {
    for cycle in DependencyGraph::from_spec(spec).cycles {
        let rendered = cycle
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" -> ");
        push_error(errors, "dependency_cycle", "dependency_graph", rendered);
    }
}

fn collect_local(
    id: &SpecId,
    path: &str,
    local: &mut BTreeSet<SpecId>,
    errors: &mut Vec<ValidationError>,
) {
    if !id.is_valid() {
        push_error(
            errors,
            "invalid_id",
            path,
            format!("noncanonical identifier '{id}'"),
        );
    }
    if !local.insert(id.clone()) {
        push_error(
            errors,
            "duplicate_local_id",
            path,
            format!("duplicate local identifier '{id}'"),
        );
    }
}

fn require_reference(
    id: &SpecId,
    valid: &BTreeSet<SpecId>,
    path: &str,
    errors: &mut Vec<ValidationError>,
) {
    if !valid.contains(id) {
        push_error(
            errors,
            "missing_reference",
            path,
            format!("unresolved reference '{id}'"),
        );
    }
}

fn push_error(errors: &mut Vec<ValidationError>, code: &str, path: &str, message: String) {
    errors.push(ValidationError {
        code: code.to_owned(),
        path: path.to_owned(),
        message,
    });
}
