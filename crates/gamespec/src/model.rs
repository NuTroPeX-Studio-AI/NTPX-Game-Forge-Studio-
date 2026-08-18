use serde::{Deserialize, Serialize};

/// Root machine-readable project specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSpec {
    pub schema_version: String,
    pub project_id: String,
    pub title: String,
    #[serde(default)]
    pub modules: Vec<ModuleSpec>,
}

/// A logical project module containing related features.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleSpec {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub features: Vec<FeatureSpec>,
}

/// A user- or system-facing capability within a module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureSpec {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub entities: Vec<EntitySpec>,
}

/// A deterministic gameplay or project entity declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySpec {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub components: Vec<String>,
}
