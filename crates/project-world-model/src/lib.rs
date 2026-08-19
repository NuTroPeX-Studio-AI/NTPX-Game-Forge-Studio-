//! Versioned project world-model contracts for NTPX GameForge Ω∞.

use std::{collections::BTreeMap, error::Error, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectEntity {
    pub id: String,
    pub kind: String,
    pub version: u64,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub relation: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub source: String,
    pub subject_id: String,
    pub state_digest_sha256: String,
    pub observed_unix: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectWorldModel {
    pub entities: BTreeMap<String, ProjectEntity>,
    pub relationships: Vec<Relationship>,
    pub observations: Vec<Observation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldModelError {
    EmptyEntityId,
    EmptyKind,
    EmptyProvenance,
    DuplicateEntity(String),
    UnknownRelationshipEndpoint(String),
    EmptyRelation,
    InvalidObservationDigest,
    UnknownObservationSubject(String),
}

impl fmt::Display for WorldModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyEntityId => formatter.write_str("world-model entity id must not be empty"),
            Self::EmptyKind => formatter.write_str("world-model entity kind must not be empty"),
            Self::EmptyProvenance => formatter.write_str("world-model provenance must not be empty"),
            Self::DuplicateEntity(id) => write!(formatter, "duplicate world-model entity '{id}'"),
            Self::UnknownRelationshipEndpoint(id) => {
                write!(formatter, "relationship references unknown entity '{id}'")
            }
            Self::EmptyRelation => formatter.write_str("relationship type must not be empty"),
            Self::InvalidObservationDigest => formatter.write_str("observation digest must be SHA-256 hex"),
            Self::UnknownObservationSubject(id) => {
                write!(formatter, "observation references unknown entity '{id}'")
            }
        }
    }
}

impl Error for WorldModelError {}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

impl ProjectWorldModel {
    /// Insert a provenance-bearing project entity.
    ///
    /// # Errors
    /// Returns [`WorldModelError`] for malformed or duplicate entities.
    pub fn insert_entity(&mut self, entity: ProjectEntity) -> Result<(), WorldModelError> {
        if entity.id.trim().is_empty() {
            return Err(WorldModelError::EmptyEntityId);
        }
        if entity.kind.trim().is_empty() {
            return Err(WorldModelError::EmptyKind);
        }
        if entity.provenance.trim().is_empty() {
            return Err(WorldModelError::EmptyProvenance);
        }
        if self.entities.insert(entity.id.clone(), entity.clone()).is_some() {
            return Err(WorldModelError::DuplicateEntity(entity.id));
        }
        Ok(())
    }

    /// Add an explicit relation whose endpoints already exist.
    ///
    /// # Errors
    /// Returns [`WorldModelError`] for unknown endpoints or empty relation type.
    pub fn add_relationship(&mut self, relationship: Relationship) -> Result<(), WorldModelError> {
        if relationship.relation.trim().is_empty() {
            return Err(WorldModelError::EmptyRelation);
        }
        for endpoint in [&relationship.from, &relationship.to] {
            if !self.entities.contains_key(endpoint) {
                return Err(WorldModelError::UnknownRelationshipEndpoint(endpoint.clone()));
            }
        }
        self.relationships.push(relationship);
        self.relationships.sort_by(|left, right| {
            (&left.from, &left.relation, &left.to).cmp(&(&right.from, &right.relation, &right.to))
        });
        Ok(())
    }

    /// Add an observation for a known project entity.
    ///
    /// # Errors
    /// Returns [`WorldModelError`] for unknown subjects or malformed digests.
    pub fn observe(&mut self, observation: Observation) -> Result<(), WorldModelError> {
        if !self.entities.contains_key(&observation.subject_id) {
            return Err(WorldModelError::UnknownObservationSubject(observation.subject_id));
        }
        if !valid_digest(&observation.state_digest_sha256) {
            return Err(WorldModelError::InvalidObservationDigest);
        }
        self.observations.push(observation);
        self.observations.sort_by(|left, right| {
            (&left.subject_id, left.observed_unix, &left.source)
                .cmp(&(&right.subject_id, right.observed_unix, &right.source))
        });
        Ok(())
    }

    #[must_use]
    pub fn snapshot_digest(&self) -> String {
        let encoded = serde_json::to_vec(self).unwrap_or_default();
        format!("{:x}", Sha256::digest(encoded))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relationships_require_known_entities() {
        let mut model = ProjectWorldModel::default();
        model
            .insert_entity(ProjectEntity {
                id: "player".to_owned(),
                kind: "gameplay_actor".to_owned(),
                version: 1,
                provenance: "gamespec".to_owned(),
            })
            .expect("valid entity");
        let error = model
            .add_relationship(Relationship {
                from: "player".to_owned(),
                relation: "uses".to_owned(),
                to: "weapon".to_owned(),
            })
            .expect_err("unknown endpoint must fail");
        assert!(matches!(error, WorldModelError::UnknownRelationshipEndpoint(_)));
    }
}
