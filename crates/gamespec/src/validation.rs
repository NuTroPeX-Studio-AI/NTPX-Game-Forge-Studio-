use crate::{GameSpec, SpecId, SUPPORTED_SCHEMA_VERSION};
use std::collections::HashSet;

/// One deterministic validation finding for a `GameSpec` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: String,
    pub path: String,
    pub message: String,
}

/// Full validation result. An empty issue list means the document is valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// Returns true when no validation issue was found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    /// Returns validation findings in deterministic discovery order.
    #[must_use]
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }
}

impl GameSpec {
    /// Validates schema compatibility, identifier integrity, uniqueness, and direct references.
    #[must_use]
    pub fn validate(&self) -> ValidationReport {
        let mut issues = Vec::new();
        let mut all_ids = HashSet::new();
        let mut module_ids = HashSet::new();
        let mut component_ids = HashSet::new();
        let mut npc_ids = HashSet::new();
        let mut test_ids = HashSet::new();

        if self.schema_version != SUPPORTED_SCHEMA_VERSION {
            push_issue(
                &mut issues,
                "unsupported_schema_version",
                "schema_version",
                format!(
                    "expected schema version {SUPPORTED_SCHEMA_VERSION}, found {}",
                    self.schema_version
                ),
            );
        }

        collect_id(&self.project.id, "project.id", &mut all_ids, &mut issues);

        for (index, module) in self.architecture.modules.iter().enumerate() {
            collect_id(
                &module.id,
                &format!("architecture.modules[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
            module_ids.insert(module.id.as_str().to_owned());
        }

        for (index, feature) in self.features.iter().enumerate() {
            collect_id(
                &feature.id,
                &format!("features[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
        }
        for (index, component) in self.components.iter().enumerate() {
            collect_id(
                &component.id,
                &format!("components[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
            component_ids.insert(component.id.as_str().to_owned());
        }
        for (index, entity) in self.entities.iter().enumerate() {
            collect_id(
                &entity.id,
                &format!("entities[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
        }
        for (index, input) in self.inputs.iter().enumerate() {
            collect_id(
                &input.id,
                &format!("inputs[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
        }
        for (index, ability) in self.abilities.iter().enumerate() {
            collect_id(
                &ability.id,
                &format!("abilities[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
        }
        for (index, npc) in self.npcs.iter().enumerate() {
            collect_id(
                &npc.id,
                &format!("npcs[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
            npc_ids.insert(npc.id.as_str().to_owned());
        }
        for (index, dialogue) in self.dialogues.iter().enumerate() {
            collect_id(
                &dialogue.id,
                &format!("dialogues[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
        }
        for (index, quest) in self.quests.iter().enumerate() {
            collect_id(
                &quest.id,
                &format!("quests[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
        }
        for (index, world) in self.worlds.iter().enumerate() {
            collect_id(
                &world.id,
                &format!("worlds[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
        }
        for (index, asset) in self.assets.iter().enumerate() {
            collect_id(
                &asset.id,
                &format!("assets[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
        }
        for (index, test) in self.tests.iter().enumerate() {
            collect_id(
                &test.id,
                &format!("tests[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
            test_ids.insert(test.id.as_str().to_owned());
        }
        for (index, build) in self.builds.iter().enumerate() {
            collect_id(
                &build.id,
                &format!("builds[{index}].id"),
                &mut all_ids,
                &mut issues,
            );
        }

        for (module_index, module) in self.architecture.modules.iter().enumerate() {
            for (dependency_index, dependency) in module.depends_on.iter().enumerate() {
                require_reference(
                    dependency,
                    &module_ids,
                    &format!(
                        "architecture.modules[{module_index}].depends_on[{dependency_index}]"
                    ),
                    "missing_module_reference",
                    &mut issues,
                );
            }
        }

        for (feature_index, feature) in self.features.iter().enumerate() {
            require_reference(
                &feature.owner_module,
                &module_ids,
                &format!("features[{feature_index}].owner_module"),
                "missing_module_reference",
                &mut issues,
            );
            for (dependency_index, dependency) in feature.dependencies.iter().enumerate() {
                require_reference(
                    dependency,
                    &all_ids,
                    &format!("features[{feature_index}].dependencies[{dependency_index}]"),
                    "missing_reference",
                    &mut issues,
                );
            }
            for (test_index, test) in feature.tests.iter().enumerate() {
                require_reference(
                    test,
                    &test_ids,
                    &format!("features[{feature_index}].tests[{test_index}]"),
                    "missing_test_reference",
                    &mut issues,
                );
            }
        }

        for (entity_index, entity) in self.entities.iter().enumerate() {
            for (component_index, component) in entity.components.iter().enumerate() {
                require_reference(
                    component,
                    &component_ids,
                    &format!("entities[{entity_index}].components[{component_index}]"),
                    "missing_component_reference",
                    &mut issues,
                );
            }
        }

        for (ability_index, ability) in self.abilities.iter().enumerate() {
            require_reference(
                &ability.owner_module,
                &module_ids,
                &format!("abilities[{ability_index}].owner_module"),
                "missing_module_reference",
                &mut issues,
            );
        }

        for (npc_index, npc) in self.npcs.iter().enumerate() {
            for (component_index, component) in npc.components.iter().enumerate() {
                require_reference(
                    component,
                    &component_ids,
                    &format!("npcs[{npc_index}].components[{component_index}]"),
                    "missing_component_reference",
                    &mut issues,
                );
            }
        }

        for (dialogue_index, dialogue) in self.dialogues.iter().enumerate() {
            let mut node_ids = HashSet::new();
            for (node_index, node) in dialogue.nodes.iter().enumerate() {
                collect_local_id(
                    &node.id,
                    &format!("dialogues[{dialogue_index}].nodes[{node_index}].id"),
                    &mut node_ids,
                    &mut issues,
                );
            }
            for (participant_index, participant) in dialogue.participants.iter().enumerate() {
                require_reference(
                    participant,
                    &npc_ids,
                    &format!(
                        "dialogues[{dialogue_index}].participants[{participant_index}]"
                    ),
                    "missing_npc_reference",
                    &mut issues,
                );
            }
            for (node_index, node) in dialogue.nodes.iter().enumerate() {
                require_reference(
                    &node.speaker,
                    &npc_ids,
                    &format!("dialogues[{dialogue_index}].nodes[{node_index}].speaker"),
                    "missing_npc_reference",
                    &mut issues,
                );
                for (next_index, next) in node.next.iter().enumerate() {
                    require_reference(
                        next,
                        &node_ids,
                        &format!(
                            "dialogues[{dialogue_index}].nodes[{node_index}].next[{next_index}]"
                        ),
                        "missing_dialogue_node_reference",
                        &mut issues,
                    );
                }
            }
        }

        for (quest_index, quest) in self.quests.iter().enumerate() {
            let mut step_ids = HashSet::new();
            for (step_index, step) in quest.steps.iter().enumerate() {
                collect_local_id(
                    &step.id,
                    &format!("quests[{quest_index}].steps[{step_index}].id"),
                    &mut step_ids,
                    &mut issues,
                );
                for (feature_index, feature) in step.requires_features.iter().enumerate() {
                    require_reference(
                        feature,
                        &all_ids,
                        &format!(
                            "quests[{quest_index}].steps[{step_index}].requires_features[{feature_index}]"
                        ),
                        "missing_reference",
                        &mut issues,
                    );
                }
            }
        }

        for (world_index, world) in self.worlds.iter().enumerate() {
            let mut region_ids = HashSet::new();
            for (region_index, region) in world.regions.iter().enumerate() {
                collect_local_id(
                    &region.id,
                    &format!("worlds[{world_index}].regions[{region_index}].id"),
                    &mut region_ids,
                    &mut issues,
                );
            }
        }

        for (test_index, test) in self.tests.iter().enumerate() {
            require_reference(
                &test.target,
                &all_ids,
                &format!("tests[{test_index}].target"),
                "missing_reference",
                &mut issues,
            );
        }

        ValidationReport { issues }
    }
}

fn collect_id(
    id: &SpecId,
    path: &str,
    all_ids: &mut HashSet<String>,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_id(id, path, issues);
    if !all_ids.insert(id.as_str().to_owned()) {
        push_issue(
            issues,
            "duplicate_id",
            path,
            format!("duplicate global id: {id}"),
        );
    }
}

fn collect_local_id(
    id: &SpecId,
    path: &str,
    local_ids: &mut HashSet<String>,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_id(id, path, issues);
    if !local_ids.insert(id.as_str().to_owned()) {
        push_issue(
            issues,
            "duplicate_local_id",
            path,
            format!("duplicate local id: {id}"),
        );
    }
}

fn validate_id(id: &SpecId, path: &str, issues: &mut Vec<ValidationIssue>) {
    if !id.is_valid() {
        push_issue(
            issues,
            "invalid_id",
            path,
            format!("noncanonical GameSpec id: {id}"),
        );
    }
}

fn require_reference(
    id: &SpecId,
    valid_ids: &HashSet<String>,
    path: &str,
    code: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if !valid_ids.contains(id.as_str()) {
        push_issue(
            issues,
            code,
            path,
            format!("unresolved reference: {id}"),
        );
    }
}

fn push_issue(
    issues: &mut Vec<ValidationIssue>,
    code: &str,
    path: &str,
    message: String,
) {
    issues.push(ValidationIssue {
        code: code.to_owned(),
        path: path.to_owned(),
        message,
    });
}
