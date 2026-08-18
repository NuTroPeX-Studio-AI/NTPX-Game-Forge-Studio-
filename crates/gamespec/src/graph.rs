use crate::{GameSpec, SpecId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Kind of directed dependency edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyEdgeKind {
    ModuleDependency,
    FeatureDependency,
    FeatureTest,
    EntityComponent,
    AbilityModule,
    NpcComponent,
    DialogueParticipant,
    DialogueSpeaker,
    DialogueNext,
    QuestFeature,
    TestTarget,
}

/// One directed reference in a compiled dependency graph.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: SpecId,
    pub to: SpecId,
    pub kind: DependencyEdgeKind,
}

/// Deterministic dependency graph extracted from a `GameSpec`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub edges: Vec<DependencyEdge>,
    pub cycles: Vec<Vec<SpecId>>,
}

impl DependencyGraph {
    /// Build a deterministic graph from direct references.
    #[must_use]
    pub fn from_spec(spec: &GameSpec) -> Self {
        let mut edges = Vec::new();
        push_module_edges(spec, &mut edges);
        push_feature_edges(spec, &mut edges);
        push_entity_edges(spec, &mut edges);
        push_ability_edges(spec, &mut edges);
        push_npc_edges(spec, &mut edges);
        push_dialogue_edges(spec, &mut edges);
        push_quest_edges(spec, &mut edges);
        push_test_edges(spec, &mut edges);
        edges.sort();
        edges.dedup();
        let cycles = find_dependency_cycles(&edges);
        Self { edges, cycles }
    }

    /// Return true when dependency edges contain no cycle.
    #[must_use]
    pub fn is_acyclic(&self) -> bool {
        self.cycles.is_empty()
    }
}

fn push_module_edges(spec: &GameSpec, edges: &mut Vec<DependencyEdge>) {
    for module in &spec.architecture.modules {
        for dependency in &module.depends_on {
            edges.push(DependencyEdge {
                from: module.id.clone(),
                to: dependency.clone(),
                kind: DependencyEdgeKind::ModuleDependency,
            });
        }
    }
}

fn push_feature_edges(spec: &GameSpec, edges: &mut Vec<DependencyEdge>) {
    for feature in &spec.features {
        for dependency in &feature.dependencies {
            edges.push(DependencyEdge {
                from: feature.id.clone(),
                to: dependency.clone(),
                kind: DependencyEdgeKind::FeatureDependency,
            });
        }
        for test in &feature.tests {
            edges.push(DependencyEdge {
                from: feature.id.clone(),
                to: test.clone(),
                kind: DependencyEdgeKind::FeatureTest,
            });
        }
    }
}

fn push_entity_edges(spec: &GameSpec, edges: &mut Vec<DependencyEdge>) {
    for entity in &spec.entities {
        for component in &entity.components {
            edges.push(DependencyEdge {
                from: entity.id.clone(),
                to: component.clone(),
                kind: DependencyEdgeKind::EntityComponent,
            });
        }
    }
}

fn push_ability_edges(spec: &GameSpec, edges: &mut Vec<DependencyEdge>) {
    for ability in &spec.abilities {
        edges.push(DependencyEdge {
            from: ability.id.clone(),
            to: ability.owner_module.clone(),
            kind: DependencyEdgeKind::AbilityModule,
        });
    }
}

fn push_npc_edges(spec: &GameSpec, edges: &mut Vec<DependencyEdge>) {
    for npc in &spec.npcs {
        for component in &npc.components {
            edges.push(DependencyEdge {
                from: npc.id.clone(),
                to: component.clone(),
                kind: DependencyEdgeKind::NpcComponent,
            });
        }
    }
}

fn push_dialogue_edges(spec: &GameSpec, edges: &mut Vec<DependencyEdge>) {
    for dialogue in &spec.dialogues {
        for participant in &dialogue.participants {
            edges.push(DependencyEdge {
                from: dialogue.id.clone(),
                to: participant.clone(),
                kind: DependencyEdgeKind::DialogueParticipant,
            });
        }
        for node in &dialogue.nodes {
            edges.push(DependencyEdge {
                from: dialogue.id.clone(),
                to: node.speaker.clone(),
                kind: DependencyEdgeKind::DialogueSpeaker,
            });
            for next in &node.next {
                edges.push(DependencyEdge {
                    from: node.id.clone(),
                    to: next.clone(),
                    kind: DependencyEdgeKind::DialogueNext,
                });
            }
        }
    }
}

fn push_quest_edges(spec: &GameSpec, edges: &mut Vec<DependencyEdge>) {
    for quest in &spec.quests {
        for step in &quest.steps {
            for feature in &step.requires_features {
                edges.push(DependencyEdge {
                    from: quest.id.clone(),
                    to: feature.clone(),
                    kind: DependencyEdgeKind::QuestFeature,
                });
            }
        }
    }
}

fn push_test_edges(spec: &GameSpec, edges: &mut Vec<DependencyEdge>) {
    for test in &spec.tests {
        edges.push(DependencyEdge {
            from: test.id.clone(),
            to: test.target.clone(),
            kind: DependencyEdgeKind::TestTarget,
        });
    }
}

fn find_dependency_cycles(edges: &[DependencyEdge]) -> Vec<Vec<SpecId>> {
    let dependency_kinds = [
        DependencyEdgeKind::ModuleDependency,
        DependencyEdgeKind::FeatureDependency,
    ];
    let mut adjacency: BTreeMap<SpecId, BTreeSet<SpecId>> = BTreeMap::new();
    for edge in edges {
        if dependency_kinds.contains(&edge.kind) {
            adjacency
                .entry(edge.from.clone())
                .or_default()
                .insert(edge.to.clone());
        }
    }

    let mut cycles = BTreeSet::new();
    for start in adjacency.keys() {
        let mut path = Vec::new();
        let mut active = BTreeSet::new();
        walk_cycles(
            start,
            start,
            &adjacency,
            &mut active,
            &mut path,
            &mut cycles,
        );
    }
    cycles.into_iter().collect()
}

fn walk_cycles(
    start: &SpecId,
    current: &SpecId,
    adjacency: &BTreeMap<SpecId, BTreeSet<SpecId>>,
    active: &mut BTreeSet<SpecId>,
    path: &mut Vec<SpecId>,
    cycles: &mut BTreeSet<Vec<SpecId>>,
) {
    if !active.insert(current.clone()) {
        return;
    }
    path.push(current.clone());
    if let Some(next_nodes) = adjacency.get(current) {
        for next in next_nodes {
            if next == start {
                let mut cycle = path.clone();
                cycle.push(start.clone());
                cycles.insert(cycle);
            } else if !active.contains(next) {
                walk_cycles(start, next, adjacency, active, path, cycles);
            }
        }
    }
    path.pop();
    active.remove(current);
}
