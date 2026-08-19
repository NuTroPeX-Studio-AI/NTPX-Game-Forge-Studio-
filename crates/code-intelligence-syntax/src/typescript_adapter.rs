use std::{error::Error, fmt, fmt::Write};

use code_intelligence::{Language, SourceFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tree_sitter::{Node, Parser};

/// TypeScript declaration kinds recognized by the AST parser adapter.
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

/// Deterministic TypeScript AST parser adapter.
///
/// The adapter uses the upstream Tree-sitter TypeScript grammar to parse the complete source file,
/// then derives supported top-level declaration evidence from parser nodes. It intentionally does
/// not claim TypeScript compiler binding, module resolution, type checking, control/data-flow
/// analysis, language-server semantics, or refactor execution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TypeScriptParserAdapter;

impl TypeScriptParserAdapter {
    /// Parse TypeScript source into deterministic AST-derived top-level declaration evidence.
    ///
    /// # Errors
    /// Returns [`TypeScriptParserError`] for an empty path, non-TypeScript input, parser setup
    /// failure, malformed TypeScript syntax, or a supported declaration whose required name is
    /// absent from the parsed syntax tree.
    pub fn parse(source: &SourceFile) -> Result<TypeScriptParseDocument, TypeScriptParserError> {
        validate_source(source)?;

        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        parser
            .set_language(&language.into())
            .map_err(|error| TypeScriptParserError::ParserInitialization(error.to_string()))?;
        let tree = parser
            .parse(&source.content, None)
            .ok_or(TypeScriptParserError::ParserUnavailable)?;
        let root = tree.root_node();
        if root.has_error() {
            return Err(TypeScriptParserError::MalformedSyntax {
                path: source.path.clone(),
                line: first_error_line(root).unwrap_or(1),
            });
        }

        let mut items = Vec::new();
        let mut cursor = root.walk();
        for child in root.named_children(&mut cursor) {
            collect_top_level(child, &source.content, &mut items)?;
        }

        let evidence_sha256 = parse_digest(&source.path, &items);
        Ok(TypeScriptParseDocument {
            path: source.path.clone(),
            items,
            evidence_sha256,
        })
    }
}

fn validate_source(source: &SourceFile) -> Result<(), TypeScriptParserError> {
    if source.path.trim().is_empty() {
        return Err(TypeScriptParserError::EmptyPath);
    }
    if source.language != Language::TypeScript {
        return Err(TypeScriptParserError::LanguageMismatch {
            path: source.path.clone(),
            actual: source.language,
        });
    }
    Ok(())
}

fn collect_top_level(
    node: Node<'_>,
    source: &str,
    items: &mut Vec<TypeScriptItem>,
) -> Result<(), TypeScriptParserError> {
    match node.kind() {
        "export_statement" | "ambient_declaration" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                collect_top_level(child, source, items)?;
            }
        }
        "function_declaration" | "generator_function_declaration" => {
            push_named_item(node, source, TypeScriptItemKind::Function, items)?;
        }
        "class_declaration" | "abstract_class_declaration" => {
            push_named_item(node, source, TypeScriptItemKind::Class, items)?;
        }
        "interface_declaration" => {
            push_named_item(node, source, TypeScriptItemKind::Interface, items)?;
        }
        "type_alias_declaration" => {
            push_named_item(node, source, TypeScriptItemKind::TypeAlias, items)?;
        }
        "enum_declaration" => {
            push_named_item(node, source, TypeScriptItemKind::Enum, items)?;
        }
        "internal_module" | "module_declaration" => {
            push_named_item(node, source, TypeScriptItemKind::Namespace, items)?;
        }
        "lexical_declaration" => collect_variable_items(node, source, items)?,
        "variable_declaration" => {
            collect_variable_items_with_kind(node, source, TypeScriptItemKind::Var, items)?;
        }
        _ => {}
    }
    Ok(())
}

fn push_named_item(
    node: Node<'_>,
    source: &str,
    kind: TypeScriptItemKind,
    items: &mut Vec<TypeScriptItem>,
) -> Result<(), TypeScriptParserError> {
    let name_node = node.child_by_field_name("name").ok_or_else(|| {
        TypeScriptParserError::MalformedDeclaration {
            line: node_line(node),
            declaration: node_text(node, source).to_owned(),
        }
    })?;
    let name = node_text(name_node, source);
    if name.trim().is_empty() {
        return Err(TypeScriptParserError::MalformedDeclaration {
            line: node_line(node),
            declaration: node_text(node, source).to_owned(),
        });
    }
    push_item(items, kind, name, node_line(name_node));
    Ok(())
}

fn collect_variable_items(
    node: Node<'_>,
    source: &str,
    items: &mut Vec<TypeScriptItem>,
) -> Result<(), TypeScriptParserError> {
    let text = node_text(node, source).trim_start();
    let kind = if text.starts_with("const") {
        TypeScriptItemKind::Const
    } else if text.starts_with("let") {
        TypeScriptItemKind::Let
    } else {
        return Err(TypeScriptParserError::MalformedDeclaration {
            line: node_line(node),
            declaration: text.to_owned(),
        });
    };
    collect_variable_items_with_kind(node, source, kind, items)
}

fn collect_variable_items_with_kind(
    node: Node<'_>,
    source: &str,
    kind: TypeScriptItemKind,
    items: &mut Vec<TypeScriptItem>,
) -> Result<(), TypeScriptParserError> {
    let mut found = false;
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() != "variable_declarator" {
            continue;
        }
        let Some(name_node) = child.child_by_field_name("name") else {
            return Err(TypeScriptParserError::MalformedDeclaration {
                line: node_line(child),
                declaration: node_text(child, source).to_owned(),
            });
        };
        let name = node_text(name_node, source);
        if !is_supported_binding_name(name) {
            continue;
        }
        push_item(items, kind, name, node_line(name_node));
        found = true;
    }
    if !found {
        return Err(TypeScriptParserError::MalformedDeclaration {
            line: node_line(node),
            declaration: node_text(node, source).to_owned(),
        });
    }
    Ok(())
}

fn is_supported_binding_name(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_alphabetic())
        && characters
            .all(|character| character == '_' || character == '$' || character.is_alphanumeric())
}

fn push_item(items: &mut Vec<TypeScriptItem>, kind: TypeScriptItemKind, name: &str, line: u32) {
    items.push(TypeScriptItem {
        kind,
        name: name.to_owned(),
        line,
        ordinal: u32::try_from(items.len()).unwrap_or(u32::MAX),
    });
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
    node.children(&mut cursor)
        .filter(|child| child.has_error() || child.is_missing())
        .find_map(first_error_line)
}

fn parse_digest(path: &str, items: &[TypeScriptItem]) -> String {
    let mut canonical = format!("{path}\nTypeScriptTreeSitter\n");
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
    LanguageMismatch { path: String, actual: Language },
    ParserInitialization(String),
    ParserUnavailable,
    MalformedSyntax { path: String, line: u32 },
    MalformedDeclaration { line: u32, declaration: String },
}

impl fmt::Display for TypeScriptParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => formatter.write_str("source path must not be empty"),
            Self::LanguageMismatch { path, actual } => write!(
                formatter,
                "source '{path}' language {actual:?} does not match TypeScript parser adapter"
            ),
            Self::ParserInitialization(reason) => {
                write!(
                    formatter,
                    "TypeScript parser initialization failed: {reason}"
                )
            }
            Self::ParserUnavailable => {
                formatter.write_str("TypeScript parser returned no syntax tree")
            }
            Self::MalformedSyntax { path, line } => write!(
                formatter,
                "source '{path}' contains malformed TypeScript syntax near line {line}"
            ),
            Self::MalformedDeclaration { line, declaration } => write!(
                formatter,
                "malformed TypeScript declaration at line {line}: {declaration}"
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
    fn ast_evidence_is_deterministic_for_supported_declarations() {
        let source = typescript_source(
            "export interface Actor {}\nexport default class Player<T> implements Actor {}\nexport async function spawn<T>(value: T): Promise<T> { return value; }\ntype Id = string;\nconst MAX_HEALTH = 100;\n",
        );

        let first = TypeScriptParserAdapter::parse(&source).expect("valid TypeScript AST");
        let second = TypeScriptParserAdapter::parse(&source).expect("valid TypeScript AST");

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
    fn multiline_declarations_use_parser_locations() {
        let document = TypeScriptParserAdapter::parse(&typescript_source(
            "interface Actor<\n  T\n> { value: T }\n\nexport class Player<\n  T extends Actor<string>\n> {}\n",
        ))
        .expect("multiline TypeScript must parse");

        assert_eq!(document.items.len(), 2);
        assert_eq!(document.items[0].name, "Actor");
        assert_eq!(document.items[0].line, 1);
        assert_eq!(document.items[1].name, "Player");
        assert_eq!(document.items[1].line, 5);
    }

    #[test]
    fn nested_declarations_are_not_promoted_to_top_level() {
        let document = TypeScriptParserAdapter::parse(&typescript_source(
            "class Player {\n  tick() {\n    const local = 1;\n  }\n}\nconst GLOBAL = 2;\n",
        ))
        .expect("valid TypeScript AST");

        assert_eq!(document.items.len(), 2);
        assert_eq!(document.items[0].name, "Player");
        assert_eq!(document.items[1].name, "GLOBAL");
    }

    #[test]
    fn regex_and_template_expressions_do_not_corrupt_ast() {
        let document = TypeScriptParserAdapter::parse(&typescript_source(
            "const PATTERN = /[{}]/g;\nconst TEMPLATE = `value ${JSON.stringify({ ok: true })}`;\nclass Player {}\n",
        ))
        .expect("valid TypeScript grammar constructs must parse");

        assert_eq!(document.items.len(), 3);
        assert_eq!(document.items[2].name, "Player");
    }

    #[test]
    fn multiple_variable_declarators_are_explicit() {
        let document = TypeScriptParserAdapter::parse(&typescript_source(
            "const FIRST = 1, SECOND = 2;\nlet THIRD = 3;\nvar FOURTH = 4;\n",
        ))
        .expect("valid variable declarations");

        let names: Vec<_> = document
            .items
            .iter()
            .map(|item| item.name.as_str())
            .collect();
        assert_eq!(names, vec!["FIRST", "SECOND", "THIRD", "FOURTH"]);
        assert_eq!(document.items[0].kind, TypeScriptItemKind::Const);
        assert_eq!(document.items[2].kind, TypeScriptItemKind::Let);
        assert_eq!(document.items[3].kind, TypeScriptItemKind::Var);
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
    fn malformed_typescript_fails_closed() {
        let error = TypeScriptParserAdapter::parse(&typescript_source(
            "export class Player<T {\n  value: T;\n}\n",
        ))
        .expect_err("malformed TypeScript must fail");

        assert!(matches!(
            error,
            TypeScriptParserError::MalformedSyntax { .. }
                | TypeScriptParserError::MalformedDeclaration { .. }
        ));
    }
}
