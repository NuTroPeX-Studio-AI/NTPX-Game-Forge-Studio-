use std::{error::Error, fmt, fmt::Write};

use code_intelligence::{Language, SourceFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tree_sitter::{Node, Parser};

/// C++ top-level declaration kinds emitted by the parser adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CppItemKind {
    Function,
    Class,
    Struct,
    Enum,
    Namespace,
    TypeAlias,
}

/// Deterministic parser-derived C++ declaration evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CppItem {
    pub kind: CppItemKind,
    pub name: String,
    pub line: u32,
    pub ordinal: u32,
}

/// AST parser output for one C++ source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CppParseDocument {
    pub path: String,
    pub items: Vec<CppItem>,
    pub evidence_sha256: String,
}

/// Tree-sitter-backed C++ syntax parser adapter.
///
/// This adapter extracts a bounded set of top-level declaration evidence. It does not claim Clang
/// compilation, preprocessing, template instantiation, overload resolution, type checking, or LSP
/// semantics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CppParserAdapter;

impl CppParserAdapter {
    /// Parse C++ source into deterministic top-level declaration evidence.
    ///
    /// # Errors
    /// Returns [`CppParserError`] for invalid language/path input, parser setup failure, malformed
    /// C++ syntax, or a recognized declaration with no usable name.
    pub fn parse(source: &SourceFile) -> Result<CppParseDocument, CppParserError> {
        validate_source(source)?;
        let mut parser = Parser::new();
        let language = tree_sitter_cpp::LANGUAGE;
        parser
            .set_language(&language.into())
            .map_err(|error| CppParserError::ParserInitialization(error.to_string()))?;
        let tree = parser
            .parse(&source.content, None)
            .ok_or(CppParserError::ParserUnavailable)?;
        let root = tree.root_node();
        if root.has_error() {
            return Err(CppParserError::MalformedSyntax {
                path: source.path.clone(),
                line: first_error_line(root).unwrap_or(1),
            });
        }

        let mut items = Vec::new();
        let mut cursor = root.walk();
        for child in root.named_children(&mut cursor) {
            collect_top_level(child, &source.content, &mut items)?;
        }

        Ok(CppParseDocument {
            path: source.path.clone(),
            evidence_sha256: parse_digest(&source.path, &items),
            items,
        })
    }
}

fn validate_source(source: &SourceFile) -> Result<(), CppParserError> {
    if source.path.trim().is_empty() {
        return Err(CppParserError::EmptyPath);
    }
    if source.language != Language::Cpp {
        return Err(CppParserError::LanguageMismatch {
            path: source.path.clone(),
            actual: source.language,
        });
    }
    Ok(())
}

fn collect_top_level(
    node: Node<'_>,
    source: &str,
    items: &mut Vec<CppItem>,
) -> Result<(), CppParserError> {
    match node.kind() {
        "function_definition" => push_function(node, source, items)?,
        "class_specifier" => push_named(node, source, CppItemKind::Class, items)?,
        "struct_specifier" => push_named(node, source, CppItemKind::Struct, items)?,
        "enum_specifier" => push_named(node, source, CppItemKind::Enum, items)?,
        "namespace_definition" => push_named(node, source, CppItemKind::Namespace, items)?,
        "alias_declaration" | "type_definition" => {
            push_named(node, source, CppItemKind::TypeAlias, items)?;
        }
        "template_declaration" | "declaration" | "linkage_specification" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                if is_top_level_candidate(child.kind()) {
                    collect_top_level(child, source, items)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn is_top_level_candidate(kind: &str) -> bool {
    matches!(
        kind,
        "function_definition"
            | "class_specifier"
            | "struct_specifier"
            | "enum_specifier"
            | "namespace_definition"
            | "alias_declaration"
            | "type_definition"
            | "template_declaration"
            | "declaration"
            | "linkage_specification"
    )
}

fn push_function(
    node: Node<'_>,
    source: &str,
    items: &mut Vec<CppItem>,
) -> Result<(), CppParserError> {
    let declarator = node
        .child_by_field_name("declarator")
        .ok_or_else(|| malformed_declaration(node, source))?;
    let name_node = find_name_node(declarator).ok_or_else(|| malformed_declaration(node, source))?;
    push_item(items, CppItemKind::Function, node_text(name_node, source), name_node);
    Ok(())
}

fn push_named(
    node: Node<'_>,
    source: &str,
    kind: CppItemKind,
    items: &mut Vec<CppItem>,
) -> Result<(), CppParserError> {
    let name_node = node
        .child_by_field_name("name")
        .or_else(|| find_name_node(node))
        .ok_or_else(|| malformed_declaration(node, source))?;
    let name = node_text(name_node, source);
    if name.trim().is_empty() {
        return Err(malformed_declaration(node, source));
    }
    push_item(items, kind, name, name_node);
    Ok(())
}

fn find_name_node(node: Node<'_>) -> Option<Node<'_>> {
    if let Some(name) = node.child_by_field_name("name") {
        return Some(name);
    }
    if matches!(
        node.kind(),
        "identifier" | "field_identifier" | "type_identifier" | "operator_name" | "destructor_name"
    ) {
        return Some(node);
    }
    let mut cursor = node.walk();
    let result = node.named_children(&mut cursor).find_map(find_name_node);
    result
}

fn push_item(items: &mut Vec<CppItem>, kind: CppItemKind, name: &str, node: Node<'_>) {
    items.push(CppItem {
        kind,
        name: name.to_owned(),
        line: node_line(node),
        ordinal: u32::try_from(items.len()).unwrap_or(u32::MAX),
    });
}

fn malformed_declaration(node: Node<'_>, source: &str) -> CppParserError {
    CppParserError::MalformedDeclaration {
        line: node_line(node),
        declaration: node_text(node, source).to_owned(),
    }
}

fn node_text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    source.get(node.byte_range()).unwrap_or_default()
}

fn node_line(node: Node<'_>) -> u32 {
    u32::try_from(node.start_position().row + 1).unwrap_or(u32::MAX)
}

fn first_error_line(node: Node<'_>) -> Option<u32> {
    if node.is_error() || node.is_missing() {
        return Some(node_line(node));
    }
    let mut cursor = node.walk();
    let result = node
        .children(&mut cursor)
        .filter(|child| child.has_error() || child.is_missing())
        .find_map(first_error_line);
    result
}

fn parse_digest(path: &str, items: &[CppItem]) -> String {
    let mut canonical = format!("{path}\nCppTreeSitter\n");
    for item in items {
        writeln!(
            &mut canonical,
            "{:?}\t{}\t{}\t{}",
            item.kind, item.name, item.line, item.ordinal
        )
        .expect("writing C++ parser evidence to String cannot fail");
    }
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

/// Fail-closed C++ parser-adapter errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CppParserError {
    EmptyPath,
    LanguageMismatch { path: String, actual: Language },
    ParserInitialization(String),
    ParserUnavailable,
    MalformedSyntax { path: String, line: u32 },
    MalformedDeclaration { line: u32, declaration: String },
}

impl fmt::Display for CppParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "C++ parser validation failed: {self:?}")
    }
}

impl Error for CppParserError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpp_source(content: &str) -> SourceFile {
        SourceFile {
            path: "Source/Game/Player.cpp".to_owned(),
            language: Language::Cpp,
            content: content.to_owned(),
            generated: false,
        }
    }

    #[test]
    fn parses_top_level_cpp_items_deterministically() {
        let source = cpp_source(
            "namespace Game {}\nclass Player {};\nstruct Stats {};\nenum class State { Idle, Run };\nusing PlayerId = unsigned long;\nint spawn(int count) { return count; }\n",
        );
        let first = CppParserAdapter::parse(&source).expect("valid C++ AST");
        let second = CppParserAdapter::parse(&source).expect("valid C++ AST");
        assert_eq!(first, second);
        let names: Vec<_> = first.items.iter().map(|item| item.name.as_str()).collect();
        assert!(names.contains(&"Game"));
        assert!(names.contains(&"Player"));
        assert!(names.contains(&"Stats"));
        assert!(names.contains(&"State"));
        assert!(names.contains(&"PlayerId"));
        assert!(names.contains(&"spawn"));
    }

    #[test]
    fn class_methods_are_not_promoted_to_top_level() {
        let document = CppParserAdapter::parse(&cpp_source(
            "class Player { public: void tick() {} };\nint global_tick() { return 0; }\n",
        ))
        .expect("valid C++ AST");
        let names: Vec<_> = document.items.iter().map(|item| item.name.as_str()).collect();
        assert!(names.contains(&"Player"));
        assert!(names.contains(&"global_tick"));
        assert!(!names.contains(&"tick"));
    }

    #[test]
    fn templates_are_unwrapped_at_top_level() {
        let document = CppParserAdapter::parse(&cpp_source(
            "template <typename T> class Box {};\ntemplate <typename T> T identity(T value) { return value; }\n",
        ))
        .expect("valid templates");
        let names: Vec<_> = document.items.iter().map(|item| item.name.as_str()).collect();
        assert!(names.contains(&"Box"));
        assert!(names.contains(&"identity"));
    }

    #[test]
    fn malformed_cpp_fails_closed() {
        let error = CppParserAdapter::parse(&cpp_source("class Player {\nint spawn( {\n"))
            .expect_err("malformed C++ must fail");
        assert!(matches!(error, CppParserError::MalformedSyntax { .. }));
    }

    #[test]
    fn language_mismatch_fails_closed() {
        let source = SourceFile {
            path: "src/player.ts".to_owned(),
            language: Language::TypeScript,
            content: "class Player {}".to_owned(),
            generated: false,
        };
        assert!(matches!(
            CppParserAdapter::parse(&source),
            Err(CppParserError::LanguageMismatch { .. })
        ));
    }
}
