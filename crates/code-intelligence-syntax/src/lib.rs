//! Deterministic provider-neutral syntax-service boundary for NTPX `GameForge` Ω∞.
//!
//! This crate is the first Phase 06 language-service substrate above lexical indexing. It emits
//! deterministic structural syntax evidence without claiming compiler/LSP-grade AST semantics.
//! It performs no source mutation, model invocation, tool execution, network access, credential
//! use, repository write, engine action, publishing, signing, or release action.

use std::{error::Error, fmt, fmt::Write};

use code_intelligence::{Language, SourceFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Structural syntax event emitted by a syntax service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyntaxNodeKind {
    Declaration,
    BlockOpen,
    BlockClose,
}

/// Stable structural syntax evidence for one source line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxNode {
    pub kind: SyntaxNodeKind,
    pub text: String,
    pub line: u32,
    pub depth: u32,
}

/// Deterministic syntax document for one source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxDocument {
    pub path: String,
    pub language: Language,
    pub nodes: Vec<SyntaxNode>,
    pub evidence_sha256: String,
}

/// Explicit provider-neutral language-service parsing boundary.
pub trait SyntaxService: Send + Sync {
    fn language(&self) -> Language;

    /// Parse one source file into deterministic structural syntax evidence.
    ///
    /// # Errors
    /// Returns [`SyntaxServiceError`] for invalid input, language mismatch, or unbalanced blocks.
    fn parse(&self, source: &SourceFile) -> Result<SyntaxDocument, SyntaxServiceError>;
}

/// Deterministic structural syntax service used to establish the language-service contract.
///
/// This service recognizes declaration prefixes and balanced brace structure. It is intentionally
/// not represented as a native compiler AST or LSP semantic parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralSyntaxService {
    language: Language,
}

impl StructuralSyntaxService {
    #[must_use]
    pub const fn new(language: Language) -> Self {
        Self { language }
    }

    const fn declaration_prefixes(self) -> &'static [&'static str] {
        match self.language {
            Language::Rust => &[
                "fn ", "struct ", "enum ", "trait ", "mod ", "impl ", "const ",
            ],
            Language::TypeScript => &[
                "function ",
                "class ",
                "interface ",
                "type ",
                "const ",
                "enum ",
            ],
            Language::Cpp => &["class ", "struct ", "enum ", "void ", "bool ", "int "],
            Language::BlueprintMetadata => &["Function=", "Class=", "Variable="],
        }
    }
}

impl SyntaxService for StructuralSyntaxService {
    fn language(&self) -> Language {
        self.language
    }

    fn parse(&self, source: &SourceFile) -> Result<SyntaxDocument, SyntaxServiceError> {
        if source.path.trim().is_empty() {
            return Err(SyntaxServiceError::EmptyPath);
        }
        if source.language != self.language {
            return Err(SyntaxServiceError::LanguageMismatch {
                path: source.path.clone(),
                expected: self.language,
                actual: source.language,
            });
        }

        let mut nodes = Vec::new();
        let mut depth = 0_u32;
        for (offset, raw_line) in source.content.lines().enumerate() {
            let line_number = u32::try_from(offset + 1).unwrap_or(u32::MAX);
            let trimmed = raw_line.trim();

            if self
                .declaration_prefixes()
                .iter()
                .any(|prefix| trimmed.starts_with(prefix))
            {
                nodes.push(SyntaxNode {
                    kind: SyntaxNodeKind::Declaration,
                    text: trimmed.to_owned(),
                    line: line_number,
                    depth,
                });
            }

            for character in trimmed.chars() {
                match character {
                    '{' => {
                        nodes.push(SyntaxNode {
                            kind: SyntaxNodeKind::BlockOpen,
                            text: "{".to_owned(),
                            line: line_number,
                            depth,
                        });
                        depth = depth.saturating_add(1);
                    }
                    '}' => {
                        depth = depth.checked_sub(1).ok_or_else(|| {
                            SyntaxServiceError::UnbalancedBlock {
                                path: source.path.clone(),
                                line: line_number,
                            }
                        })?;
                        nodes.push(SyntaxNode {
                            kind: SyntaxNodeKind::BlockClose,
                            text: "}".to_owned(),
                            line: line_number,
                            depth,
                        });
                    }
                    _ => {}
                }
            }
        }

        if depth != 0 {
            return Err(SyntaxServiceError::UnbalancedBlock {
                path: source.path.clone(),
                line: u32::try_from(source.content.lines().count()).unwrap_or(u32::MAX),
            });
        }

        let evidence_sha256 = syntax_digest(&source.path, source.language, &nodes);
        Ok(SyntaxDocument {
            path: source.path.clone(),
            language: source.language,
            nodes,
            evidence_sha256,
        })
    }
}

/// Syntax-service validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxServiceError {
    EmptyPath,
    LanguageMismatch {
        path: String,
        expected: Language,
        actual: Language,
    },
    UnbalancedBlock {
        path: String,
        line: u32,
    },
}

impl fmt::Display for SyntaxServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => formatter.write_str("source path must not be empty"),
            Self::LanguageMismatch {
                path,
                expected,
                actual,
            } => write!(
                formatter,
                "source '{path}' language {actual:?} does not match syntax service {expected:?}"
            ),
            Self::UnbalancedBlock { path, line } => {
                write!(
                    formatter,
                    "source '{path}' has unbalanced block structure at line {line}"
                )
            }
        }
    }
}

impl Error for SyntaxServiceError {}

fn syntax_digest(path: &str, language: Language, nodes: &[SyntaxNode]) -> String {
    let mut canonical = format!("{path}\n{language:?}\n");
    for node in nodes {
        writeln!(
            &mut canonical,
            "{:?}\t{}\t{}\t{}",
            node.kind, node.line, node.depth, node.text
        )
        .expect("writing syntax evidence to String cannot fail");
    }
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

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
    fn structural_evidence_is_deterministic() {
        let service = StructuralSyntaxService::new(Language::Rust);
        let source = rust_source("struct Player {\nfn tick() {\n}\n}");

        let first = service.parse(&source).expect("valid structure");
        let second = service.parse(&source).expect("valid structure");

        assert_eq!(first, second);
        assert_eq!(first.nodes.len(), 6);
        assert_eq!(first.nodes[0].kind, SyntaxNodeKind::Declaration);
        assert_eq!(first.nodes[1].kind, SyntaxNodeKind::BlockOpen);
    }

    #[test]
    fn language_mismatch_fails_closed() {
        let service = StructuralSyntaxService::new(Language::TypeScript);
        let error = service
            .parse(&rust_source("fn tick() {}"))
            .expect_err("mismatched language must fail");

        assert!(matches!(error, SyntaxServiceError::LanguageMismatch { .. }));
    }

    #[test]
    fn unmatched_close_fails_closed() {
        let service = StructuralSyntaxService::new(Language::Rust);
        let error = service
            .parse(&rust_source("fn tick() }"))
            .expect_err("unmatched close must fail");

        assert!(matches!(error, SyntaxServiceError::UnbalancedBlock { .. }));
    }

    #[test]
    fn unmatched_open_fails_closed() {
        let service = StructuralSyntaxService::new(Language::Rust);
        let error = service
            .parse(&rust_source("fn tick() {"))
            .expect_err("unmatched open must fail");

        assert!(matches!(error, SyntaxServiceError::UnbalancedBlock { .. }));
    }
}
