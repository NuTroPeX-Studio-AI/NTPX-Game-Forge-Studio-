use std::{error::Error, fmt, fmt::Write};

use code_intelligence::{Language, SourceFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use syn::{spanned::Spanned, Item, Type};

/// Rust item kinds recognized by the compiler-grade syntax adapter.
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

/// Deterministic top-level Rust item evidence derived from a parsed Rust syntax tree.
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

/// Deterministic Rust-specific parser adapter backed by `syn`'s full Rust parser.
///
/// This adapter parses the complete source file before emitting supported top-level item evidence,
/// so comments, strings, nested braces, generics, attributes, and ordinary Rust syntax do not rely
/// on line-prefix heuristics. It remains a syntax boundary: it does not provide macro expansion,
/// name/type resolution, control-flow analysis, a true call graph, or LSP semantics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RustParserAdapter;

impl RustParserAdapter {
    /// Parse Rust source into deterministic top-level syntax evidence.
    ///
    /// # Errors
    /// Returns [`RustParserError`] for an empty path, non-Rust input, or a Rust parse failure.
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

        let file = syn::parse_file(&source.content).map_err(|error| RustParserError::ParseFailure {
            path: source.path.clone(),
            line: span_line(error.span()),
            message: error.to_string(),
        })?;

        let items = file
            .items
            .iter()
            .filter_map(item_evidence)
            .enumerate()
            .map(|(ordinal, mut item)| {
                item.ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
                item
            })
            .collect::<Vec<_>>();

        let evidence_sha256 = parse_digest(&source.path, &items);
        Ok(RustParseDocument {
            path: source.path.clone(),
            items,
            evidence_sha256,
        })
    }
}

fn item_evidence(item: &Item) -> Option<RustItem> {
    let (kind, name) = match item {
        Item::Fn(item) => (RustItemKind::Function, item.sig.ident.to_string()),
        Item::Struct(item) => (RustItemKind::Struct, item.ident.to_string()),
        Item::Enum(item) => (RustItemKind::Enum, item.ident.to_string()),
        Item::Trait(item) => (RustItemKind::Trait, item.ident.to_string()),
        Item::Mod(item) => (RustItemKind::Module, item.ident.to_string()),
        Item::Impl(item) => (RustItemKind::Impl, impl_target_name(&item.self_ty)),
        Item::Type(item) => (RustItemKind::TypeAlias, item.ident.to_string()),
        Item::Const(item) => (RustItemKind::Const, item.ident.to_string()),
        Item::Static(item) => (RustItemKind::Static, item.ident.to_string()),
        _ => return None,
    };

    Some(RustItem {
        kind,
        name,
        line: span_line(item.span()),
        ordinal: 0,
    })
}

fn impl_target_name(target: &Type) -> String {
    if let Type::Path(type_path) = target {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident.to_string();
        }
    }
    "<impl-target>".to_owned()
}

fn span_line(span: proc_macro2::Span) -> u32 {
    u32::try_from(span.start().line).unwrap_or(u32::MAX)
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
    LanguageMismatch {
        path: String,
        actual: Language,
    },
    ParseFailure {
        path: String,
        line: u32,
        message: String,
    },
}

impl fmt::Display for RustParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => formatter.write_str("source path must not be empty"),
            Self::LanguageMismatch { path, actual } => write!(
                formatter,
                "source '{path}' language {actual:?} does not match Rust parser adapter"
            ),
            Self::ParseFailure {
                path,
                line,
                message,
            } => write!(
                formatter,
                "source '{path}' failed Rust syntax parsing at line {line}: {message}"
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
            "#[derive(Debug)]\npub struct Player<T> {\n    health: T,\n}\n\nimpl<T> Player<T> {\n    fn tick(&mut self) {}\n}\n\npub(crate) fn spawn() {}\n",
        );

        let first = RustParserAdapter::parse(&source).expect("valid Rust syntax");
        let second = RustParserAdapter::parse(&source).expect("valid Rust syntax");

        assert_eq!(first, second);
        assert_eq!(first.items.len(), 3);
        assert_eq!(first.items[0].kind, RustItemKind::Struct);
        assert_eq!(first.items[0].name, "Player");
        assert_eq!(first.items[0].line, 1);
        assert_eq!(first.items[1].kind, RustItemKind::Impl);
        assert_eq!(first.items[1].name, "Player");
        assert_eq!(first.items[2].name, "spawn");
    }

    #[test]
    fn ignores_nested_method_as_top_level_item() {
        let document = RustParserAdapter::parse(&rust_source(
            "struct Player {}\nimpl Player {\n    fn tick(&mut self) {}\n}\n",
        ))
        .expect("valid Rust syntax");

        assert_eq!(document.items.len(), 2);
        assert!(document.items.iter().all(|item| item.name != "tick"));
    }

    #[test]
    fn parser_handles_braces_inside_comments_and_strings() {
        let document = RustParserAdapter::parse(&rust_source(
            "// unmatched-looking brace: {\nconst LABEL: &str = \"}\";\nfn spawn() { println!(\"{{}}\"); }\n",
        ))
        .expect("parser must understand Rust lexical structure");

        assert_eq!(document.items.len(), 2);
        assert_eq!(document.items[0].name, "LABEL");
        assert_eq!(document.items[1].name, "spawn");
    }

    #[test]
    fn supports_rust_items_that_span_multiple_lines() {
        let document = RustParserAdapter::parse(&rust_source(
            "pub fn spawn<\n    T: Default,\n>(value: T)\nwhere\n    T: Clone,\n{\n    let _ = value;\n}\n",
        ))
        .expect("multiline Rust item must parse");

        assert_eq!(document.items.len(), 1);
        assert_eq!(document.items[0].kind, RustItemKind::Function);
        assert_eq!(document.items[0].name, "spawn");
        assert_eq!(document.items[0].line, 1);
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
    fn malformed_rust_fails_closed_with_parser_evidence() {
        let error = RustParserAdapter::parse(&rust_source("pub struct {\n}\n"))
            .expect_err("malformed Rust must fail");

        assert!(matches!(
            error,
            RustParserError::ParseFailure { line: 1, .. }
        ));
    }

    #[test]
    fn unsupported_top_level_items_are_ignored_after_full_parse() {
        let document = RustParserAdapter::parse(&rust_source(
            "use std::fmt;\nmacro_rules! marker { () => {} }\nstruct Player;\n",
        ))
        .expect("valid Rust syntax");

        assert_eq!(document.items.len(), 1);
        assert_eq!(document.items[0].name, "Player");
    }
}
