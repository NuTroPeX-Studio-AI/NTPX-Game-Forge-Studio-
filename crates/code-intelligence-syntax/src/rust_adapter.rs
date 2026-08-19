use std::{error::Error, fmt, fmt::Write};

use code_intelligence::{Language, SourceFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Rust item kinds recognized by the deterministic native adapter foundation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustItemKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Impl,
    TypeAlias,
    Const,
    Static,
}

/// Deterministic top-level Rust item evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustItem {
    pub kind: RustItemKind,
    pub name: String,
    pub line: u32,
    pub ordinal: u32,
}

/// Parser output for one Rust source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustParseDocument {
    pub path: String,
    pub items: Vec<RustItem>,
    pub evidence_sha256: String,
}

/// Deterministic Rust-specific parser adapter foundation.
///
/// This adapter recognizes top-level Rust item declarations while respecting brace depth. It is
/// deliberately narrower than a `rustc` AST and does not provide type resolution, macro expansion,
/// control-flow analysis, or LSP semantics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RustParserAdapter;

impl RustParserAdapter {
    /// Parse top-level Rust item declarations into deterministic evidence.
    ///
    /// # Errors
    /// Returns [`RustParserError`] for empty paths, non-Rust input, malformed recognized item
    /// declarations, or unbalanced brace structure.
    pub fn parse(source: &SourceFile) -> Result<RustParseDocument, RustParserError> {
        if source.path.trim().is_empty() {
            return Err(RustParserError::EmptyPath);
        }
        if source.language != Language::Rust {
            return Err(RustParserError::LanguageMismatch {
                path: source.path.clone(),
                actual: source.language,
            });
        }

        let mut items = Vec::new();
        let mut depth = 0_u32;
        for (offset, raw_line) in source.content.lines().enumerate() {
            let line = u32::try_from(offset + 1).unwrap_or(u32::MAX);
            let trimmed = raw_line.trim();

            if depth == 0 {
                if let Some((kind, remainder)) = recognized_prefix(trimmed) {
                    let name = item_name(kind, remainder).ok_or_else(|| {
                        RustParserError::MalformedItem {
                            path: source.path.clone(),
                            line,
                        }
                    })?;
                    items.push(RustItem {
                        kind,
                        name,
                        line,
                        ordinal: u32::try_from(items.len()).unwrap_or(u32::MAX),
                    });
                }
            }

            for character in trimmed.chars() {
                match character {
                    '{' => depth = depth.saturating_add(1),
                    '}' => {
                        depth = depth.checked_sub(1).ok_or_else(|| {
                            RustParserError::UnbalancedBlock {
                                path: source.path.clone(),
                                line,
                            }
                        })?;
                    }
                    _ => {}
                }
            }
        }

        if depth != 0 {
            return Err(RustParserError::UnbalancedBlock {
                path: source.path.clone(),
                line: u32::try_from(source.content.lines().count()).unwrap_or(u32::MAX),
            });
        }

        let evidence_sha256 = parse_digest(&source.path, &items);
        Ok(RustParseDocument {
            path: source.path.clone(),
            items,
            evidence_sha256,
        })
    }
}

fn recognized_prefix(line: &str) -> Option<(RustItemKind, &str)> {
    const PREFIXES: [(&str, RustItemKind); 9] = [
        ("fn ", RustItemKind::Function),
        ("struct ", RustItemKind::Struct),
        ("enum ", RustItemKind::Enum),
        ("trait ", RustItemKind::Trait),
        ("mod ", RustItemKind::Module),
        ("impl ", RustItemKind::Impl),
        ("type ", RustItemKind::TypeAlias),
        ("const ", RustItemKind::Const),
        ("static ", RustItemKind::Static),
    ];

    let line = strip_visibility(line);
    PREFIXES
        .iter()
        .find_map(|(prefix, kind)| line.strip_prefix(prefix).map(|rest| (*kind, rest)))
}

fn strip_visibility(line: &str) -> &str {
    if let Some(rest) = line.strip_prefix("pub ") {
        return rest;
    }
    if let Some(rest) = line.strip_prefix("pub(crate) ") {
        return rest;
    }
    if let Some(rest) = line.strip_prefix("pub(super) ") {
        return rest;
    }
    line
}

fn item_name(kind: RustItemKind, remainder: &str) -> Option<String> {
    if kind == RustItemKind::Impl {
        let candidate = remainder
            .split(['{', '<'])
            .next()?
            .trim()
            .split_whitespace()
            .last()?;
        return valid_identifier(candidate).then(|| candidate.to_owned());
    }

    let candidate = remainder
        .split(|character: char| {
            character.is_whitespace()
                || matches!(character, '(' | '<' | '{' | ':' | '=' | ';')
        })
        .next()?
        .trim();
    valid_identifier(candidate).then(|| candidate.to_owned())
}

fn valid_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn parse_digest(path: &str, items: &[RustItem]) -> String {
    let mut canonical = format!("{path}\nRust\n");
    for item in items {
        writeln!(
            &mut canonical,
            "{:?}\t{}\t{}\t{}",
            item.kind, item.name, item.line, item.ordinal
        )
        .expect("writing Rust parser evidence to String cannot fail");
    }
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

/// Fail-closed Rust parser adapter errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustParserError {
    EmptyPath,
    LanguageMismatch { path: String, actual: Language },
    MalformedItem { path: String, line: u32 },
    UnbalancedBlock { path: String, line: u32 },
}

impl fmt::Display for RustParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => formatter.write_str("source path must not be empty"),
            Self::LanguageMismatch { path, actual } => write!(
                formatter,
                "source '{path}' language {actual:?} does not match Rust parser adapter"
            ),
            Self::MalformedItem { path, line } => {
                write!(formatter, "source '{path}' has malformed Rust item at line {line}")
            }
            Self::UnbalancedBlock { path, line } => write!(
                formatter,
                "source '{path}' has unbalanced Rust block structure at line {line}"
            ),
        }
    }
}

impl Error for RustParserError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn rust_source(content: &str) -> SourceFile {
        SourceFile {
            path: "src/player.rs".to_owned(),
            language: Language::Rust,
            content: content.to_owned(),
            generated: false,
        }
    }

    #[test]
    fn parses_top_level_items_deterministically() {
        let source = rust_source(
            "pub struct Player {\n    health: u32,\n}\n\nimpl Player {\n    fn tick(&mut self) {}\n}\n\npub(crate) fn spawn() {}\n",
        );

        let first = RustParserAdapter::parse(&source).expect("valid Rust structure");
        let second = RustParserAdapter::parse(&source).expect("valid Rust structure");

        assert_eq!(first, second);
        assert_eq!(first.items.len(), 3);
        assert_eq!(first.items[0].kind, RustItemKind::Struct);
        assert_eq!(first.items[0].name, "Player");
        assert_eq!(first.items[1].kind, RustItemKind::Impl);
        assert_eq!(first.items[2].name, "spawn");
    }

    #[test]
    fn ignores_nested_method_as_top_level_item() {
        let document = RustParserAdapter::parse(&rust_source(
            "struct Player {}\nimpl Player {\n    fn tick(&mut self) {}\n}\n",
        ))
        .expect("valid Rust structure");

        assert_eq!(document.items.len(), 2);
        assert!(document.items.iter().all(|item| item.name != "tick"));
    }

    #[test]
    fn language_mismatch_fails_closed() {
        let source = SourceFile {
            path: "src/player.ts".to_owned(),
            language: Language::TypeScript,
            content: "export class Player {}".to_owned(),
            generated: false,
        };

        assert!(matches!(
            RustParserAdapter::parse(&source),
            Err(RustParserError::LanguageMismatch { .. })
        ));
    }

    #[test]
    fn malformed_recognized_item_fails_closed() {
        let error = RustParserAdapter::parse(&rust_source("pub struct {\n}\n"))
            .expect_err("malformed item must fail");
        assert!(matches!(error, RustParserError::MalformedItem { .. }));
    }

    #[test]
    fn unbalanced_structure_fails_closed() {
        let error = RustParserAdapter::parse(&rust_source("fn tick() {\n"))
            .expect_err("unbalanced structure must fail");
        assert!(matches!(error, RustParserError::UnbalancedBlock { .. }));
    }
}
