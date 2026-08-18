use crate::SpecId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Root canonical game specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSpec {
    pub schema_version: String,
    pub project: ProjectSpec,
    pub architecture: ArchitectureSpec,
    #[serde(default)]
    pub features: Vec<FeatureSpec>,
    #[serde(default)]
    pub components: Vec<ComponentSpec>,
    #[serde(default)]
    pub entities: Vec<EntitySpec>,
    #[serde(default)]
    pub inputs: Vec<InputSpec>,
    #[serde(default)]
    pub abilities: Vec<AbilitySpec>,
    #[serde(default)]
    pub npcs: Vec<NpcSpec>,
    #[serde(default)]
    pub dialogues: Vec<DialogueSpec>,
    #[serde(default)]
    pub quests: Vec<QuestSpec>,
    #[serde(default)]
    pub worlds: Vec<WorldSpec>,
    #[serde(default)]
    pub assets: Vec<AssetSpec>,
    #[serde(default)]
    pub tests: Vec<TestSpec>,
    #[serde(default)]
    pub builds: Vec<BuildSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSpec {
    pub id: SpecId,
    pub title: String,
    pub slug: String,
    pub description: String,
    #[serde(default)]
    pub engine_targets: Vec<EngineTarget>,
    #[serde(default)]
    pub platform_targets: Vec<PlatformTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureSpec {
    #[serde(default)]
    pub modules: Vec<ModuleSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleSpec {
    pub id: SpecId,
    pub name: String,
    pub kind: ModuleKind,
    #[serde(default)]
    pub depends_on: Vec<SpecId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleKind {
    Core,
    Gameplay,
    Ai,
    Narrative,
    World,
    Ui,
    Audio,
    Tooling,
    Adapter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureSpec {
    pub id: SpecId,
    pub name: String,
    pub owner_module: SpecId,
    #[serde(default)]
    pub requirements: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<SpecId>,
    #[serde(default)]
    pub tests: Vec<SpecId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentSpec {
    pub id: SpecId,
    pub name: String,
    #[serde(default)]
    pub data: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySpec {
    pub id: SpecId,
    pub name: String,
    #[serde(default)]
    pub components: Vec<SpecId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputSpec {
    pub id: SpecId,
    pub name: String,
    pub value_type: InputValueType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputValueType {
    Digital,
    Axis1d,
    Axis2d,
    Axis3d,
    Voice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilitySpec {
    pub id: SpecId,
    pub name: String,
    pub owner_module: SpecId,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NpcSpec {
    pub id: SpecId,
    pub display_name: String,
    #[serde(default)]
    pub components: Vec<SpecId>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogueSpec {
    pub id: SpecId,
    pub title: String,
    #[serde(default)]
    pub participants: Vec<SpecId>,
    #[serde(default)]
    pub nodes: Vec<DialogueNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: SpecId,
    pub speaker: SpecId,
    pub text: String,
    #[serde(default)]
    pub next: Vec<SpecId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestSpec {
    pub id: SpecId,
    pub title: String,
    #[serde(default)]
    pub steps: Vec<QuestStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestStep {
    pub id: SpecId,
    pub objective: String,
    #[serde(default)]
    pub requires_features: Vec<SpecId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSpec {
    pub id: SpecId,
    pub name: String,
    #[serde(default)]
    pub regions: Vec<WorldRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldRegion {
    pub id: SpecId,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetSpec {
    pub id: SpecId,
    pub name: String,
    pub kind: AssetKind,
    pub source_uri: String,
    pub license: AssetLicense,
    pub provenance: AssetProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Mesh,
    Texture,
    Material,
    Animation,
    Audio,
    Video,
    Script,
    Data,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetLicense {
    pub name: String,
    pub commercial_use: bool,
    pub attribution: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetProvenance {
    pub origin: AssetOrigin,
    pub generator: Option<String>,
    pub source_reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetOrigin {
    Human,
    Generated,
    Licensed,
    Scanned,
    Imported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestSpec {
    pub id: SpecId,
    pub name: String,
    pub kind: TestKind,
    pub target: SpecId,
    #[serde(default)]
    pub acceptance: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestKind {
    Unit,
    Integration,
    Functional,
    Performance,
    Security,
    Accessibility,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildSpec {
    pub id: SpecId,
    pub name: String,
    pub platform: PlatformTarget,
    pub profile: BuildProfile,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildProfile {
    Debug,
    Development,
    Test,
    Performance,
    ReleaseCandidate,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineTarget {
    Unreal,
    Unity,
    Godot,
    Web,
    Ntpx,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformTarget {
    Windows,
    Linux,
    Macos,
    Android,
    Ios,
    Playstation,
    Xbox,
    SteamDeck,
    Web,
    Vr,
}
