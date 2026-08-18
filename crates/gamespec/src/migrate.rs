use crate::{
    deterministic_id, validate, ArchitectureSpec, ComponentSpec, EntitySpec, FeatureSpec, GameSpec,
    GameSpecError, ModuleKind, ModuleSpec, ProjectMetadata, SpecId, CURRENT_SCHEMA_VERSION,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Deserialize)]
struct LegacyProject {
    schema_version: String,
    project_id: String,
    title: String,
    #[serde(default)]
    modules: Vec<LegacyModule>,
}

#[derive(Debug, Deserialize)]
struct LegacyModule {
    id: String,
    name: String,
    #[serde(default)]
    features: Vec<LegacyFeature>,
}

#[derive(Debug, Deserialize)]
struct LegacyFeature {
    id: String,
    name: String,
    #[serde(default)]
    entities: Vec<LegacyEntity>,
}

#[derive(Debug, Deserialize)]
struct LegacyEntity {
    id: String,
    name: String,
    #[serde(default)]
    components: Vec<String>,
}

/// Migrate legacy Phase 02 JSON into the current schema and return stable pretty JSON.
///
/// # Errors
/// Returns a decode, migration, validation, or encode error when conversion cannot complete safely.
pub fn migrate_json(input: &str) -> Result<String, GameSpecError> {
    let value: Value = serde_json::from_str(input).map_err(GameSpecError::Decode)?;
    let version = value
        .get("schema_version")
        .and_then(Value::as_str)
        .ok_or_else(|| GameSpecError::Migration("missing schema_version".to_owned()))?;

    if version == CURRENT_SCHEMA_VERSION {
        let spec: GameSpec = serde_json::from_value(value).map_err(GameSpecError::Decode)?;
        validate(&spec)?;
        return serde_json::to_string_pretty(&spec).map_err(GameSpecError::Encode);
    }

    if version != "0.1" {
        return Err(GameSpecError::Migration(format!(
            "unsupported migration source version {version}"
        )));
    }

    let legacy: LegacyProject = serde_json::from_value(value).map_err(GameSpecError::Decode)?;
    if legacy.schema_version != "0.1" {
        return Err(GameSpecError::Migration("legacy version mismatch".to_owned()));
    }
    let spec = migrate_legacy(legacy)?;
    validate(&spec)?;
    serde_json::to_string_pretty(&spec).map_err(GameSpecError::Encode)
}

fn migrate_legacy(legacy: LegacyProject) -> Result<GameSpec, GameSpecError> {
    let project_id = canonical_or_generated("project", &legacy.project_id);
    let mut modules = Vec::new();
    let mut features = Vec::new();
    let mut entities = Vec::new();
    let mut component_names = BTreeSet::new();

    for module in legacy.modules {
        let module_id = canonical_or_generated("module", &module.id);
        for feature in module.features {
            let feature_id = canonical_or_generated("feature", &feature.id);
            for entity in feature.entities {
                let component_ids = entity
                    .components
                    .iter()
                    .map(|name| {
                        component_names.insert(name.clone());
                        canonical_or_generated("component", name)
                    })
                    .collect();
                entities.push(EntitySpec {
                    id: canonical_or_generated("entity", &entity.id),
                    name: entity.name,
                    components: component_ids,
                });
            }
            features.push(FeatureSpec {
                id: feature_id,
                name: feature.name,
                owner_module: module_id.clone(),
                requirements: Vec::new(),
                dependencies: Vec::new(),
                tests: Vec::new(),
            });
        }
        modules.push(ModuleSpec {
            id: module_id,
            name: module.name,
            kind: ModuleKind::Gameplay,
            depends_on: Vec::new(),
        });
    }

    let components = component_names
        .into_iter()
        .map(|name| ComponentSpec {
            id: canonical_or_generated("component", &name),
            name,
            data: BTreeMap::new(),
        })
        .collect();

    Ok(GameSpec {
        schema_version: CURRENT_SCHEMA_VERSION.to_owned(),
        project: ProjectMetadata {
            id: project_id,
            title: legacy.title,
            slug: legacy.project_id,
            description: "Migrated from GameSpec 0.1".to_owned(),
            engine_targets: Vec::new(),
            platform_targets: Vec::new(),
        },
        architecture: ArchitectureSpec { modules },
        features,
        components,
        entities,
        inputs: Vec::new(),
        abilities: Vec::new(),
        npcs: Vec::new(),
        dialogues: Vec::new(),
        quests: Vec::new(),
        worlds: Vec::new(),
        assets: Vec::new(),
        tests: Vec::new(),
        builds: Vec::new(),
    })
}

fn canonical_or_generated(namespace: &str, value: &str) -> SpecId {
    SpecId::parse(value.to_lowercase()).unwrap_or_else(|_| deterministic_id(namespace, value))
}
