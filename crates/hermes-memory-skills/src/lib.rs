//! Local-first memory and skill contracts for NTPX GameForge Ω∞.

use std::{collections::BTreeMap, error::Error, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Working,
    Episodic,
    Semantic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub kind: MemoryKind,
    pub content: String,
    pub provenance: String,
    pub privacy_label: String,
    pub created_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillDescriptor {
    pub id: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalMemoryStore {
    records: BTreeMap<String, MemoryRecord>,
    skills: BTreeMap<String, SkillDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryError {
    EmptyId,
    EmptyContent,
    EmptyProvenance,
    EmptyPrivacyLabel,
    DuplicateRecord(String),
    DuplicateSkill(String),
    EmptySkillVersion,
    UnknownSkillDependency { skill: String, dependency: String },
}

impl fmt::Display for MemoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => formatter.write_str("memory/skill id must not be empty"),
            Self::EmptyContent => formatter.write_str("memory content must not be empty"),
            Self::EmptyProvenance => formatter.write_str("memory provenance must not be empty"),
            Self::EmptyPrivacyLabel => formatter.write_str("memory privacy label must not be empty"),
            Self::DuplicateRecord(id) => write!(formatter, "duplicate memory record '{id}'"),
            Self::DuplicateSkill(id) => write!(formatter, "duplicate skill '{id}'"),
            Self::EmptySkillVersion => formatter.write_str("skill version must not be empty"),
            Self::UnknownSkillDependency { skill, dependency } => {
                write!(formatter, "skill '{skill}' depends on unknown skill '{dependency}'")
            }
        }
    }
}

impl Error for MemoryError {}

impl LocalMemoryStore {
    /// Insert one provenance-bearing local memory record.
    ///
    /// # Errors
    /// Returns [`MemoryError`] for malformed or duplicate records.
    pub fn insert_memory(&mut self, record: MemoryRecord) -> Result<(), MemoryError> {
        if record.id.trim().is_empty() {
            return Err(MemoryError::EmptyId);
        }
        if record.content.trim().is_empty() {
            return Err(MemoryError::EmptyContent);
        }
        if record.provenance.trim().is_empty() {
            return Err(MemoryError::EmptyProvenance);
        }
        if record.privacy_label.trim().is_empty() {
            return Err(MemoryError::EmptyPrivacyLabel);
        }
        if self.records.insert(record.id.clone(), record.clone()).is_some() {
            return Err(MemoryError::DuplicateRecord(record.id));
        }
        Ok(())
    }

    /// Register a versioned skill after dependency validation.
    ///
    /// # Errors
    /// Returns [`MemoryError`] for malformed, duplicate, or unresolved skill dependencies.
    pub fn register_skill(&mut self, skill: SkillDescriptor) -> Result<(), MemoryError> {
        if skill.id.trim().is_empty() {
            return Err(MemoryError::EmptyId);
        }
        if skill.version.trim().is_empty() {
            return Err(MemoryError::EmptySkillVersion);
        }
        for dependency in &skill.dependencies {
            if !self.skills.contains_key(dependency) {
                return Err(MemoryError::UnknownSkillDependency {
                    skill: skill.id.clone(),
                    dependency: dependency.clone(),
                });
            }
        }
        if self.skills.insert(skill.id.clone(), skill.clone()).is_some() {
            return Err(MemoryError::DuplicateSkill(skill.id));
        }
        Ok(())
    }

    #[must_use]
    pub fn memory_digest(record: &MemoryRecord) -> String {
        let mut hasher = Sha256::new();
        hasher.update(record.id.as_bytes());
        hasher.update(record.content.as_bytes());
        hasher.update(record.provenance.as_bytes());
        hasher.update(record.privacy_label.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_requires_provenance_and_privacy() {
        let mut store = LocalMemoryStore::default();
        let error = store
            .insert_memory(MemoryRecord {
                id: "m1".to_owned(),
                kind: MemoryKind::Semantic,
                content: "fact".to_owned(),
                provenance: String::new(),
                privacy_label: "project".to_owned(),
                created_unix: 1,
            })
            .expect_err("missing provenance must fail");
        assert_eq!(error, MemoryError::EmptyProvenance);
    }

    #[test]
    fn skill_dependencies_are_explicit() {
        let mut store = LocalMemoryStore::default();
        let error = store
            .register_skill(SkillDescriptor {
                id: "repair".to_owned(),
                version: "1.0.0".to_owned(),
                dependencies: vec!["compile".to_owned()],
            })
            .expect_err("unknown dependency must fail");
        assert!(matches!(error, MemoryError::UnknownSkillDependency { .. }));
    }
}
