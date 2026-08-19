use std::{error::Error, fmt, fmt::Write};

use code_intelligence::{Language, SourceFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Supported Blueprint metadata declaration kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintItemKind {
    Class,
    Function,
    Variable,
}

/// Supported Blueprint metadata relationship kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintReferenceKind {
    ParentClass,
    Asset,
}

/// Deterministic Blueprint metadata declaration evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintItem {
    pub kind: BlueprintItemKind,
    pub name: String,
    pub line: u32,
    pub ordinal: u32,
}

/// Deterministic Blueprint metadata reference evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintReference {
    pub kind: BlueprintReferenceKind,
    pub target: String,
    pub line: u32,
    pub ordinal: u32,
}

/// Parsed Blueprint metadata document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintParseDocument {
    pub path: String,
    pub items: Vec<BlueprintItem>,
    pub references: Vec<BlueprintReference>,
    pub evidence_sha256: String,
}

/// Parser for the engine-neutral textual Blueprint metadata interchange boundary.
///
/// This is not a parser for Unreal's binary `.uasset` format. Binary asset inspection remains an
/// Unreal Engine adapter responsibility in Phase 08.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BlueprintMetadataParserAdapter;

impl BlueprintMetadataParserAdapter {
    /// Parse deterministic textual Blueprint metadata.
    ///
    /// Recognized records are `Class=`, `Function=`, `Variable=`, `ParentClass=`, and `Asset=`.
    /// Blank lines and `#` comments are ignored.
    ///
    /// # Errors
    /// Returns [`BlueprintParserError`] for empty paths, language mismatch, unknown record kinds,
    /// malformed records, or empty record values.
    pub fn parse(source: &SourceFile) -> Result<BlueprintParseDocument, BlueprintParserError> {
        validate_source(source)?;
        let mut items = Vec::new();
        let mut references = Vec::new();

        for (offset, raw_line) in source.content.lines().enumerate() {
            let line = u32::try_from(offset + 1).unwrap_or(u32::MAX);
            let text = raw_line.trim();
            if text.is_empty() || text.starts_with('#') {
                continue;
            }
            let Some((key, value)) = text.split_once('=') else {
                return Err(BlueprintParserError::MalformedRecord {
                    line,
                    record: text.to_owned(),
                });
            };
            let key = key.trim();
            let value = value.trim();
            if value.is_empty() {
                return Err(BlueprintParserError::EmptyValue {
                    line,
                    key: key.to_owned(),
                });
            }
            match key {
                "Class" => push_item(&mut items, BlueprintItemKind::Class, value, line),
                "Function" => push_item(&mut items, BlueprintItemKind::Function, value, line),
                "Variable" => push_item(&mut items, BlueprintItemKind::Variable, value, line),
                "ParentClass" => push_reference(
                    &mut references,
                    BlueprintReferenceKind::ParentClass,
                    value,
                    line,
                ),
                "Asset" => {
                    push_reference(&mut references, BlueprintReferenceKind::Asset, value, line)
                }
                _ => {
                    return Err(BlueprintParserError::UnknownRecord {
                        line,
                        key: key.to_owned(),
                    });
                }
            }
        }

        Ok(BlueprintParseDocument {
            path: source.path.clone(),
            evidence_sha256: parse_digest(&source.path, &items, &references),
            items,
            references,
        })
    }
}

fn validate_source(source: &SourceFile) -> Result<(), BlueprintParserError> {
    if source.path.trim().is_empty() {
        return Err(BlueprintParserError::EmptyPath);
    }
    if source.language != Language::BlueprintMetadata {
        return Err(BlueprintParserError::LanguageMismatch {
            path: source.path.clone(),
            actual: source.language,
        });
    }
    Ok(())
}

fn push_item(items: &mut Vec<BlueprintItem>, kind: BlueprintItemKind, name: &str, line: u32) {
    items.push(BlueprintItem {
        kind,
        name: name.to_owned(),
        line,
        ordinal: u32::try_from(items.len()).unwrap_or(u32::MAX),
    });
}

fn push_reference(
    references: &mut Vec<BlueprintReference>,
    kind: BlueprintReferenceKind,
    target: &str,
    line: u32,
) {
    references.push(BlueprintReference {
        kind,
        target: target.to_owned(),
        line,
        ordinal: u32::try_from(references.len()).unwrap_or(u32::MAX),
    });
}

fn parse_digest(path: &str, items: &[BlueprintItem], references: &[BlueprintReference]) -> String {
    let mut canonical = format!("{path}\nBlueprintMetadata\n");
    for item in items {
        writeln!(
            &mut canonical,
            "item\t{:?}\t{}\t{}\t{}",
            item.kind, item.name, item.line, item.ordinal
        )
        .expect("writing Blueprint item evidence to String cannot fail");
    }
    for reference in references {
        writeln!(
            &mut canonical,
            "reference\t{:?}\t{}\t{}\t{}",
            reference.kind, reference.target, reference.line, reference.ordinal
        )
        .expect("writing Blueprint reference evidence to String cannot fail");
    }
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

/// Fail-closed Blueprint metadata parser errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintParserError {
    EmptyPath,
    LanguageMismatch { path: String, actual: Language },
    MalformedRecord { line: u32, record: String },
    EmptyValue { line: u32, key: String },
    UnknownRecord { line: u32, key: String },
}

impl fmt::Display for BlueprintParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Blueprint metadata validation failed: {self:?}")
    }
}

impl Error for BlueprintParserError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn blueprint_source(content: &str) -> SourceFile {
        SourceFile {
            path: "Content/Player.blueprint.meta".to_owned(),
            language: Language::BlueprintMetadata,
            content: content.to_owned(),
            generated: false,
        }
    }

    #[test]
    fn parses_items_and_references_deterministically() {
        let source = blueprint_source(
            "# generated metadata\nClass=BP_Player\nParentClass=/Script/Game.PlayerBase\nFunction=Interact\nVariable=Health\nAsset=/Game/Characters/PlayerMesh\n",
        );
        let first = BlueprintMetadataParserAdapter::parse(&source).expect("valid metadata");
        let second = BlueprintMetadataParserAdapter::parse(&source).expect("valid metadata");
        assert_eq!(first, second);
        assert_eq!(first.items.len(), 3);
        assert_eq!(first.references.len(), 2);
        assert_eq!(first.items[0].name, "BP_Player");
        assert_eq!(first.references[0].target, "/Script/Game.PlayerBase");
    }

    #[test]
    fn unknown_record_fails_closed() {
        let error = BlueprintMetadataParserAdapter::parse(&blueprint_source("Unknown=Value\n"))
            .expect_err("unknown metadata record must fail");
        assert!(matches!(error, BlueprintParserError::UnknownRecord { .. }));
    }

    #[test]
    fn empty_value_fails_closed() {
        let error = BlueprintMetadataParserAdapter::parse(&blueprint_source("Class=\n"))
            .expect_err("empty values must fail");
        assert!(matches!(error, BlueprintParserError::EmptyValue { .. }));
    }

    #[test]
    fn language_mismatch_fails_closed() {
        let source = SourceFile {
            path: "Source/Player.cpp".to_owned(),
            language: Language::Cpp,
            content: "class Player {};".to_owned(),
            generated: false,
        };
        assert!(matches!(
            BlueprintMetadataParserAdapter::parse(&source),
            Err(BlueprintParserError::LanguageMismatch { .. })
        ));
    }
}
