use std::collections::BTreeSet;

use crate::{EntitySpec, FeatureSpec, ModuleSpec, ProjectSpec, ValidationError};

pub(crate) fn validate(spec: &ProjectSpec) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    require_non_empty(&spec.schema_version, "schema_version", &mut errors);
    require_non_empty(&spec.project_id, "project_id", &mut errors);
    require_non_empty(&spec.title, "title", &mut errors);

    if spec.schema_version != "0.1" {
        errors.push(ValidationError::new(
            "schema_version",
            format!("unsupported schema version '{}'", spec.schema_version),
        ));
    }

    validate_modules(&spec.modules, &mut errors);
    errors
}

fn validate_modules(modules: &[ModuleSpec], errors: &mut Vec<ValidationError>) {
    let mut ids = BTreeSet::new();
    for (index, module) in modules.iter().enumerate() {
        let path = format!("modules[{index}]");
        require_non_empty(&module.id, &format!("{path}.id"), errors);
        require_non_empty(&module.name, &format!("{path}.name"), errors);
        require_unique(&mut ids, &module.id, &format!("{path}.id"), errors);
        validate_features(&module.features, &path, errors);
    }
}

fn validate_features(features: &[FeatureSpec], parent: &str, errors: &mut Vec<ValidationError>) {
    let mut ids = BTreeSet::new();
    for (index, feature) in features.iter().enumerate() {
        let path = format!("{parent}.features[{index}]");
        require_non_empty(&feature.id, &format!("{path}.id"), errors);
        require_non_empty(&feature.name, &format!("{path}.name"), errors);
        require_unique(&mut ids, &feature.id, &format!("{path}.id"), errors);
        validate_entities(&feature.entities, &path, errors);
    }
}

fn validate_entities(entities: &[EntitySpec], parent: &str, errors: &mut Vec<ValidationError>) {
    let mut ids = BTreeSet::new();
    for (index, entity) in entities.iter().enumerate() {
        let path = format!("{parent}.entities[{index}]");
        require_non_empty(&entity.id, &format!("{path}.id"), errors);
        require_non_empty(&entity.name, &format!("{path}.name"), errors);
        require_unique(&mut ids, &entity.id, &format!("{path}.id"), errors);

        let mut components = BTreeSet::new();
        for (component_index, component) in entity.components.iter().enumerate() {
            let component_path = format!("{path}.components[{component_index}]");
            require_non_empty(component, &component_path, errors);
            require_unique(&mut components, component, &component_path, errors);
        }
    }
}

fn require_non_empty(value: &str, path: &str, errors: &mut Vec<ValidationError>) {
    if value.trim().is_empty() {
        errors.push(ValidationError::new(path, "must not be empty"));
    }
}

fn require_unique(
    values: &mut BTreeSet<String>,
    value: &str,
    path: &str,
    errors: &mut Vec<ValidationError>,
) {
    if !value.is_empty() && !values.insert(value.to_owned()) {
        errors.push(ValidationError::new(
            path,
            format!("duplicate identifier '{value}'"),
        ));
    }
}
