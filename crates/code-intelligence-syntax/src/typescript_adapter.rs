use std::{error::Error, fmt, fmt::Write};

use code_intelligence::{Language, SourceFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// TypeScript declaration kinds recognized by the parser-adapter foundation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeScriptItemKind {
    Function,
    Class,
    Interface,
    TypeAlias,
    Enum,
    Namespace,
    Const,
    Let,
    Var,
}

/// Deterministic top-level TypeScript declaration evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeScriptItem {
    pub kind: TypeScriptItemKind,
    pub name: String,
    pub line: u32,
    pub ordinal: u32,
}

/// Parser-adapter output for one TypeScript source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeScriptParseDocument {
    pub path: String,
    pub items: Vec<TypeScriptItem>,
    pub evidence_sha256: String,
}

/// Deterministic TypeScript-specific parser-adapter foundation.
///
/// This adapter establishes a language-specific boundary above the generic structural syntax
/// service. It recognizes supported top-level declarations while lexically shielding braces inside
/// comments and quoted/template strings. It is intentionally not represented as a TypeScript
/// compiler AST, language service, type checker, semantic resolver, or refactoring engine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TypeScriptParserAdapter;

impl TypeScriptParserAdapter {
    /// Parse TypeScript source into deterministic top-level declaration evidence.
    ///
    /// # Errors
    /// Returns [`TypeScriptParserError`] for an empty path, non-TypeScript input, malformed
    /// recognized declarations, or unbalanced brace structure outside comments and strings.
    pub fn parse(source: &SourceFile) -> Result<TypeScriptParseDocument, TypeScriptParserError> {
        if source.path.trim().is_empty() {
            return Err(TypeScriptParserError::EmptyPath);
        }
        if source.language != Language::TypeScript {
            return Err(TypeScriptParserError::LanguageMismatch {
                path: source.path.clone(),
                actual: source.language,
            });
        }

        let mut items = Vec::new();
        let mut state = LexState::default();
        let mut depth = 0_u32;

        for (offset, raw_line) in source.content.lines().enumerate() {
            let line = u32::try_from(offset + 1).unwrap_or(u32::MAX);
            if depth == 0 && !state.in_block_comment {
                if let Some(item) = parse_top_level_declaration(raw_line, line, items.len())? {
                    items.push(item);
                }
            }
            depth = scan_structure(raw_line, line, depth, &mut state).map_err(|line| {
                TypeScriptParserError::UnbalancedStructure {
                    path: source.path.clone(),
                    line,
                }
            })?;
        }

        if depth != 0 || state.in_block_comment {
            return Err(TypeScriptParserError::UnbalancedStructure {
                path: source.path.clone(),
                line: u32::try_from(source.content.lines().count()).unwrap_or(u32::MAX),
            });
        }

        let evidence_sha256 = parse_digest(&source.path, &items);
        Ok(TypeScriptParseDocument {
            path: source.path.clone(),
            items,
            evidence_sha256,
        })
    }
}

fn parse_top_level_declaration(
    raw_line: &str,
    line: u32,
    ordinal: usize,
) -> Result<Option<TypeScriptItem>, TypeScriptParserError> {
    let code = strip_leading_modifiers(raw_line.trim_start());
    let Some((kind, remainder)) = declaration_prefix(code) else {
        return Ok(None);
    };

    let name = extract_identifier(remainder).ok_or_else(|| TypeScriptParserError::MalformedDeclaration {
        line,
        declaration: raw_line.trim().to_owned(),
    })?;

    Ok(Some(TypeScriptItem {
        kind,
        name: name.to_owned(),
        line,
        ordinal: u32::try_from(ordinal).unwrap_or(u32::MAX),
    }))
}

fn strip_leading_modifiers(mut source: &str) -> &str {
    const MODIFIERS: [&str; 6] = ["export", "default", "declare", "abstract", "async", "public"];

    loop {
        let trimmed = source.trim_start();
        let Some((token, remainder)) = split_token(trimmed) else {
            return trimmed;
        };
        if MODIFIERS.contains(&token) {
            source = remainder;
        } else {
            return trimmed;
        }
    }
}

fn declaration_prefix(source: &str) -> Option<(TypeScriptItemKind, &str)> {
    const PREFIXES: [(&str, TypeScriptItemKind); 9] = [
        ("function", TypeScriptItemKind::Function),
        ("class", TypeScriptItemKind::Class),
        ("interface", TypeScriptItemKind::Interface),
        ("type", TypeScriptItemKind::TypeAlias),
        ("enum", TypeScriptItemKind::Enum),
        ("namespace", TypeScriptItemKind::Namespace),
        ("const", TypeScriptItemKind::Const),
        ("let", TypeScriptItemKind::Let),
        ("var", TypeScriptItemKind::Var),
    ];

    PREFIXES.iter().find_map(|(prefix, kind)| {
        source
            .strip_prefix(prefix)
            .filter(|remainder| remainder.starts_with(char::is_whitespace) || remainder.starts_with('*'))
            .map(|remainder| (*kind, remainder.trim_start_matches('*').trim_start()))
    })
}

fn split_token(source: &str) -> Option<(&str, &str)> {
    let end = source
        .char_indices()
        .find_map(|(index, character)| character.is_whitespace().then_some(index))?;
    Some((&source[..end], &source[end..]))
}

fn extract_identifier(source: &str) -> Option<&str> {
    let source = source.trim_start();
    let end = source
        .char_indices()
        .find_map(|(index, character)| (!is_identifier_continue(character)).then_some(index))
        .unwrap_or(source.len());
    let candidate = &source[..end];
    let first = candidate.chars().next()?;
    (is_identifier_start(first) && candidate.chars().all(is_identifier_continue)).then_some(candidate)
}

const fn is_identifier_start(character: char) -> bool {
    character == '_' || character == '$' || character.is_ascii_alphabetic()
}

const fn is_identifier_continue(character: char) -> bool {
    is_identifier_start(character) || character.is_ascii_digit()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct LexState {
    in_block_comment: bool,
}

fn scan_structure(
    line_text: &str,
    line_number: u32,
    mut depth: u32,
    state: &mut LexState,
) -> Result<u32, u32> {
    let mut characters = line_text.chars().peekable();
    let mut quote = None;
    let mut escaped = false;

    while let Some(character) = characters.next() {
        if state.in_block_comment {
            if character == '*' && characters.peek() == Some(&'/') {
                let _ = characters.next();
                state.in_block_comment = false;
            }
            continue;
        }

        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == active_quote {
                quote = None;
            }
            continue;
        }

        match character {
            '/' if characters.peek() == Some(&'/') => break,
            '/' if characters.peek() == Some(&'*') => {
                let _ = characters.next();
                state.in_block_comment = true;
            }
            '\'' | '"' | '`' => quote = Some(character),
            '{' => depth = depth.saturating_add(1),
            '}' => depth = depth.checked_sub(1).ok_or(line_number)?,
            _ => {}
        }
    }

    Ok(depth)
}

fn parse_digest(path: &str, items: &[TypeScriptItem]) -> String {
    let mut canonical = format!("{path}\nTypeScript\n");
    for item in items {
        writeln!(
            &mut canonical,
            "{:?}\t{}\t{}\t{}",
            item.kind, item.name, item.line, item.ordinal
        )
        .expect("writing TypeScript parser evidence to String cannot fail");
    }
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

/// Fail-closed TypeScript parser-adapter errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeScriptParserError {
    EmptyPath,
    LanguageMismatch {
        path: String,
        actual: Language,
    },
    MalformedDeclaration {
        line: u32,
        declaration: String,
    },
    UnbalancedStructure {
        path: String,
        line: u32,
    },
}

impl fmt::Display for TypeScriptParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => formatter.write_str("source path must not be empty"),
            Self::LanguageMismatch { path, actual } => write!(
                formatter,
                "source '{path}' language {actual:?} does not match TypeScript parser adapter"
            ),
            Self::MalformedDeclaration { line, declaration } => write!(
                formatter,
                "malformed TypeScript declaration at line {line}: {declaration}"
            ),
            Self::UnbalancedStructure { path, line } => write!(
                formatter,
                "source '{path}' has unbalanced TypeScript structure at line {line}"
            ),
        }
    }
}

impl Error for TypeScriptParserError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn typescript_source(content: &str) -> SourceFile {
        SourceFile {
            path: "src/player.ts".to_owned(),
            language: Language::TypeScript,
            content: content.to_owned(),
            generated: false,
        }
    }

    #[test]
    fn parses_supported_top_level_declarations_deterministically() {
        let source = typescript_source(
            "export interface Actor {}\nexport default class Player implements Actor {}\nexport async function spawn() {}\ntype Id = string;\nconst MAX_HEALTH = 100;\n",
        );

        let first = TypeScriptParserAdapter::parse(&source).expect("valid TypeScript structure");
        let second = TypeScriptParserAdapter::parse(&source).expect("valid TypeScript structure");

        assert_eq!(first, second);
        assert_eq!(first.items.len(), 5);
        assert_eq!(first.items[0].kind, TypeScriptItemKind::Interface);
        assert_eq!(first.items[0].name, "Actor");
        assert_eq!(first.items[1].kind, TypeScriptItemKind::Class);
        assert_eq!(first.items[1].name, "Player");
        assert_eq!(first.items[2].kind, TypeScriptItemKind::Function);
        assert_eq!(first.items[2].name, "spawn");
    }

    #[test]
    fn nested_declarations_are_not_promoted_to_top_level() {
        let document = TypeScriptParserAdapter::parse(&typescript_source(
            "class Player {\n  tick() {\n    const local = 1;\n  }\n}\nconst GLOBAL = 2;\n",
        ))
        .expect("valid TypeScript structure");

        assert_eq!(document.items.len(), 2);
        assert_eq!(document.items[0].name, "Player");
        assert_eq!(document.items[1].name, "GLOBAL");
    }

    #[test]
    fn braces_inside_comments_and_strings_do_not_change_depth() {
        let document = TypeScriptParserAdapter::parse(&typescript_source(
            "// unmatched-looking brace {\nconst LABEL = \"}\";\nconst TEMPLATE = `value { still text }`;\nclass Player {}\n",
        ))
        .expect("lexical shielding must preserve structure");

        assert_eq!(document.items.len(), 3);
        assert_eq!(document.items[2].name, "Player");
    }

    #[test]
    fn block_comments_do_not_emit_declarations() {
        let document = TypeScriptParserAdapter::parse(&typescript_source(
            "/*\nclass Hidden {}\n*/\nclass Visible {}\n",
        ))
        .expect("block comments must be ignored");

        assert_eq!(document.items.len(), 1);
        assert_eq!(document.items[0].name, "Visible");
    }

    #[test]
    fn language_mismatch_fails_closed() {
        let source = SourceFile {
            path: "src/player.rs".to_owned(),
            language: Language::Rust,
            content: "struct Player;".to_owned(),
            generated: false,
        };

        assert!(matches!(
            TypeScriptParserAdapter::parse(&source),
            Err(TypeScriptParserError::LanguageMismatch { .. })
        ));
    }

    #[test]
    fn malformed_recognized_declaration_fails_closed() {
        let error = TypeScriptParserAdapter::parse(&typescript_source("export class {\n}\n"))
            .expect_err("recognized malformed declaration must fail");

        assert!(matches!(
            error,
            TypeScriptParserError::MalformedDeclaration { line: 1, .. }
        ));
    }

    #[test]
    fn unmatched_close_fails_closed() {
        let error = TypeScriptParserAdapter::parse(&typescript_source("const value = 1;\n}\n"))
            .expect_err("unmatched close must fail");

        assert!(matches!(
            error,
            TypeScriptParserError::UnbalancedStructure { line: 2, .. }
        ));
    }

    #[test]
    fn unmatched_open_fails_closed() {
        let error = TypeScriptParserAdapter::parse(&typescript_source("class Player {\n"))
            .expect_err("unmatched open must fail");

        assert!(matches!(
            error,
            TypeScriptParserError::UnbalancedStructure { line: 1, .. }
        ));
    }
}
