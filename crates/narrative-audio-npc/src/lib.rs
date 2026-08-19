//! Narrative, audio, and NPC production contracts for NTPX GameForge Ω∞.

use std::{collections::BTreeMap, error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub id: String,
    pub display_name: String,
    pub persona: String,
    pub voice_profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: String,
    pub speaker_id: String,
    pub text: String,
    pub next: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestNode {
    pub id: String,
    pub objective: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioIntent {
    pub id: String,
    pub adapter_id: String,
    pub category: String,
    pub locale: String,
    pub lip_sync_required: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NarrativeGraph {
    pub characters: BTreeMap<String, CharacterProfile>,
    pub dialogue: BTreeMap<String, DialogueNode>,
    pub quests: BTreeMap<String, QuestNode>,
}

pub trait VoiceAdapter: Send + Sync {
    fn adapter_id(&self) -> &str;
    fn locales(&self) -> &[String];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NarrativeError {
    EmptyId,
    EmptyCharacterField,
    DuplicateCharacter(String),
    UnknownSpeaker(String),
    EmptyDialogue,
    DuplicateDialogue(String),
    UnknownDialogueLink(String),
    DuplicateQuest(String),
    UnknownQuestDependency(String),
    EmptyObjective,
    AudioAdapterMismatch,
    UnsupportedLocale,
}

impl fmt::Display for NarrativeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "narrative/audio/NPC validation failed: {self:?}")
    }
}

impl Error for NarrativeError {}

impl NarrativeGraph {
    /// Insert a character profile.
    ///
    /// # Errors
    /// Returns [`NarrativeError`] for malformed or duplicate characters.
    pub fn add_character(&mut self, character: CharacterProfile) -> Result<(), NarrativeError> {
        if character.id.trim().is_empty() {
            return Err(NarrativeError::EmptyId);
        }
        if character.display_name.trim().is_empty()
            || character.persona.trim().is_empty()
            || character.voice_profile.trim().is_empty()
        {
            return Err(NarrativeError::EmptyCharacterField);
        }
        if self
            .characters
            .insert(character.id.clone(), character.clone())
            .is_some()
        {
            return Err(NarrativeError::DuplicateCharacter(character.id));
        }
        Ok(())
    }

    /// Insert a dialogue node after speaker/link validation.
    ///
    /// # Errors
    /// Returns [`NarrativeError`] for unknown speakers/links, empty text, or duplicates.
    pub fn add_dialogue(&mut self, node: DialogueNode) -> Result<(), NarrativeError> {
        if node.id.trim().is_empty() {
            return Err(NarrativeError::EmptyId);
        }
        if !self.characters.contains_key(&node.speaker_id) {
            return Err(NarrativeError::UnknownSpeaker(node.speaker_id));
        }
        if node.text.trim().is_empty() {
            return Err(NarrativeError::EmptyDialogue);
        }
        for next in &node.next {
            if next == &node.id {
                return Err(NarrativeError::UnknownDialogueLink(next.clone()));
            }
        }
        if self
            .dialogue
            .insert(node.id.clone(), node.clone())
            .is_some()
        {
            return Err(NarrativeError::DuplicateDialogue(node.id));
        }
        Ok(())
    }

    /// Insert a quest whose dependencies already exist.
    ///
    /// # Errors
    /// Returns [`NarrativeError`] for malformed, duplicate, or unresolved quests.
    pub fn add_quest(&mut self, quest: QuestNode) -> Result<(), NarrativeError> {
        if quest.id.trim().is_empty() {
            return Err(NarrativeError::EmptyId);
        }
        if quest.objective.trim().is_empty() {
            return Err(NarrativeError::EmptyObjective);
        }
        for dependency in &quest.dependencies {
            if !self.quests.contains_key(dependency) {
                return Err(NarrativeError::UnknownQuestDependency(dependency.clone()));
            }
        }
        if self
            .quests
            .insert(quest.id.clone(), quest.clone())
            .is_some()
        {
            return Err(NarrativeError::DuplicateQuest(quest.id));
        }
        Ok(())
    }
}

/// Validate an audio/voice intent against an explicit adapter.
///
/// # Errors
/// Returns [`NarrativeError`] for adapter substitution or unsupported locale.
pub fn validate_audio_intent(
    intent: &AudioIntent,
    adapter: &dyn VoiceAdapter,
) -> Result<(), NarrativeError> {
    if intent.adapter_id != adapter.adapter_id() {
        return Err(NarrativeError::AudioAdapterMismatch);
    }
    if !adapter.locales().contains(&intent.locale) {
        return Err(NarrativeError::UnsupportedLocale);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue_requires_known_speaker() {
        let mut graph = NarrativeGraph::default();
        let error = graph
            .add_dialogue(DialogueNode {
                id: "d1".to_owned(),
                speaker_id: "missing".to_owned(),
                text: "hello".to_owned(),
                next: Vec::new(),
            })
            .expect_err("unknown speaker must fail");
        assert!(matches!(error, NarrativeError::UnknownSpeaker(_)));
    }
}
