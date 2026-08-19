//! Deterministic, provider-neutral code intelligence substrate for NTPX `GameForge` Ω∞.
//!
//! This crate indexes repository source through explicit language drivers and emits stable symbols,
//! references, change-impact evidence, and a reproducible repository digest. It performs no code
//! mutation, model invocation, tool execution, network access, credential use, or engine action.

use std::{collections::BTreeMap, error::Error, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod dependency;
mod resolution;
pub use dependency::{build_dependency_graph, DependencyEdge, DependencyGraph};
pub use resolution::{
    build_reference_graph, ReferenceGraph, ReferenceResolution, ResolvedReferenceRecord,
};

/// Language class handled by a code-intelligence driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Rust,
    TypeScript,
    Cpp,
    BlueprintMetadata,
}

/// Repository source input supplied to a language driver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub language: Language,
    pub content: String,
    pub generated: bool,
}

/// Stable symbol discovered in one source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub id: String,
    pub path: String,
    pub name: String,
    pub line: u32,
}

/// Stable lexical dependency/reference discovered in one source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceRecord {
    pub from_path: String,
    pub target: String,
    pub line: u32,
}

/// Deterministic index for one source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileIndex {
    pub path: String,
    pub language: Language,
    pub content_sha256: String,
    pub generated: bool,
    pub symbols: Vec<SymbolRecord>,
    pub references: Vec<ReferenceRecord>,
}

/// Deterministic repository index and correlation digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryIndex {
    pub files: BTreeMap<String, FileIndex>,
    pub evidence_sha256: String,
}

/// Advisory change-impact output. It does not authorize edits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeImpact {
    pub changed_path: String,
    pub dependent_paths: Vec<String>,
}

/// Explicit source-language indexing boundary.
pub trait LanguageDriver: Send + Sync {
    fn language(&self) -> Language;

    /// Index one source file.
    ///
    /// # Errors
    /// Returns [`CodeIntelligenceError`] when the source is malformed or does not match this driver.
    fn index(&self, source: &SourceFile) -> Result<FileIndex, CodeIntelligenceError>;
}

/// Deterministic lexical driver used as the first Phase 06 parsing substrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexicalLanguageDriver {
    language: Language,
}

impl LexicalLanguageDriver {
    #[must_use]
    pub const fn rust() -> Self {
        Self {
            language: Language::Rust,
        }
    }

    #[must_use]
    pub const fn typescript() -> Self {
        Self {
            language: Language::TypeScript,
        }
    }

    #[must_use]
    pub const fn cpp() -> Self {
        Self {
            language: Language::Cpp,
        }
    }

    #[must_use]
    pub const fn blueprint_metadata() -> Self {
        Self {
            language: Language::BlueprintMetadata,
        }
    }

    const fn symbol_prefixes(self) -> &'static [&'static str] {
        match self.language {
            Language::Rust => &["fn ", "struct ", "enum ", "trait ", "mod ", "const "],
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

    const fn reference_prefixes(self) -> &'static [&'static str] {
        match self.language {
            Language::Rust => &["use ", "mod "],
            Language::TypeScript => &["import ", "export "],
            Language::Cpp => &["#include "],
            Language::BlueprintMetadata => &["ParentClass=", "Asset="],
        }
    }
}

impl LanguageDriver for LexicalLanguageDriver {
    fn language(&self) -> Language {
        self.language
    }

    fn index(&self, source: &SourceFile) -> Result<FileIndex, CodeIntelligenceError> {
        if source.path.trim().is_empty() {
            return Err(CodeIntelligenceError::EmptyPath);
        }
        if source.language != self.language {
            return Err(CodeIntelligenceError::LanguageMismatch {
                path: source.path.clone(),
                expected: self.language,
                actual: source.language,
            });
        }

        let mut symbols = Vec::new();
        let mut references = Vec::new();
        for (offset, raw_line) in source.content.lines().enumerate() {
            let line = raw_line.trim();
            let line_number = u32::try_from(offset + 1).unwrap_or(u32::MAX);
            if let Some(name) = extract_prefixed_token(line, self.symbol_prefixes()) {
                symbols.push(SymbolRecord {
                    id: stable_symbol_id(&source.path, &name, line_number),
                    path: source.path.clone(),
                    name,
                    line: line_number,
                });
            }
            if let Some(target) = extract_prefixed_token(line, self.reference_prefixes()) {
                references.push(ReferenceRecord {
                    from_path: source.path.clone(),
                    target,
                    line: line_number,
                });
            }
        }
        symbols.sort_by(|left, right| left.id.cmp(&right.id));
        references
            .sort_by(|left, right| (&left.target, left.line).cmp(&(&right.target, right.line)));

        Ok(FileIndex {
            path: source.path.clone(),
            language: source.language,
            content_sha256: sha256_hex(source.content.as_bytes()),
            generated: source.generated,
            symbols,
            references,
        })
    }
}

/// Code-intelligence validation or evidence failure.
#[derive(Debug)]
pub enum CodeIntelligenceError {
    EmptyPath,
    DuplicatePath(String),
    MissingDriver(Language),
    LanguageMismatch {
        path: String,
        expected: Language,
        actual: Language,
    },
    Encode(serde_json::Error),
}

impl fmt::Display for CodeIntelligenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => formatter.write_str("source path must not be empty"),
            Self::DuplicatePath(path) => write!(formatter, "duplicate source path '{path}'"),
            Self::MissingDriver(language) => write!(formatter, "missing driver for {language:?}"),
            Self::LanguageMismatch {
                path,
                expected,
                actual,
            } => write!(
                formatter,
                "source '{path}' language {actual:?} does not match driver {expected:?}"
            ),
            Self::Encode(error) => write!(formatter, "repository index encoding failed: {error}"),
        }
    }
}

impl Error for CodeIntelligenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::EmptyPath
            | Self::DuplicatePath(_)
            | Self::MissingDriver(_)
            | Self::LanguageMismatch { .. } => None,
        }
    }
}

fn extract_prefixed_token(line: &str, prefixes: &[&str]) -> Option<String> {
    prefixes.iter().find_map(|prefix| {
        line.strip_prefix(prefix).and_then(|remainder| {
            let token = remainder
                .trim_start()
                .split(|character: char| {
                    character.is_whitespace()
                        || matches!(character, '(' | '{' | ':' | ';' | '<' | '=' | '"' | '\'')
                })
                .next()
                .unwrap_or_default()
                .trim_matches(['<', '>', '"', '\'', ';']);
            (!token.is_empty()).then(|| token.to_owned())
        })
    })
}

fn stable_symbol_id(path: &str, name: &str, line: u32) -> String {
    sha256_hex(format!("{path}\n{name}\n{line}").as_bytes())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn repository_digest(files: &BTreeMap<String, FileIndex>) -> Result<String, CodeIntelligenceError> {
    let encoded = serde_json::to_vec(files).map_err(CodeIntelligenceError::Encode)?;
    Ok(sha256_hex(&encoded))
}

/// Build a deterministic repository index using exact language-driver matching.
///
/// Input enumeration order does not affect the resulting file order or evidence digest.
///
/// # Errors
/// Returns [`CodeIntelligenceError`] for empty/duplicate paths, missing drivers, driver mismatches,
/// or evidence serialization failure.
pub fn build_repository_index(
    sources: &[SourceFile],
    drivers: &[&dyn LanguageDriver],
) -> Result<RepositoryIndex, CodeIntelligenceError> {
    let mut files = BTreeMap::new();
    for source in sources {
        if source.path.trim().is_empty() {
            return Err(CodeIntelligenceError::EmptyPath);
        }
        let driver = drivers
            .iter()
            .find(|driver| driver.language() == source.language)
            .ok_or(CodeIntelligenceError::MissingDriver(source.language))?;
        let file_index = driver.index(source)?;
        if files.insert(source.path.clone(), file_index).is_some() {
            return Err(CodeIntelligenceError::DuplicatePath(source.path.clone()));
        }
    }
    let evidence_sha256 = repository_digest(&files)?;
    Ok(RepositoryIndex {
        files,
        evidence_sha256,
    })
}

/// Find indexed files whose lexical references mention the changed path.
#[must_use]
pub fn change_impact(index: &RepositoryIndex, changed_path: &str) -> ChangeImpact {
    let mut dependent_paths: Vec<_> = index
        .files
        .values()
        .filter(|file| {
            file.references
                .iter()
                .any(|reference| reference.target.contains(changed_path))
        })
        .map(|file| file.path.clone())
        .collect();
    dependent_paths.sort();
    dependent_paths.dedup();
    ChangeImpact {
        changed_path: changed_path.to_owned(),
        dependent_paths,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sources() -> Vec<SourceFile> {
        vec![
            SourceFile {
                path: "src/lib.rs".to_owned(),
                language: Language::Rust,
                content: "use crate::core;\nfn run() {}".to_owned(),
                generated: false,
            },
            SourceFile {
                path: "ui/app.ts".to_owned(),
                language: Language::TypeScript,
                content: "import core from './core';\nfunction render() {}".to_owned(),
                generated: false,
            },
        ]
    }

    #[test]
    fn equivalent_input_order_produces_same_evidence() {
        let rust = LexicalLanguageDriver::rust();
        let typescript = LexicalLanguageDriver::typescript();
        let first = sources();
        let mut second = first.clone();
        second.reverse();

        let first_index =
            build_repository_index(&first, &[&rust, &typescript]).expect("valid index");
        let second_index =
            build_repository_index(&second, &[&typescript, &rust]).expect("valid index");

        assert_eq!(first_index, second_index);
        assert_eq!(first_index.files["src/lib.rs"].symbols[0].name, "run");
    }

    #[test]
    fn duplicate_paths_fail_closed() {
        let rust = LexicalLanguageDriver::rust();
        let source = SourceFile {
            path: "src/lib.rs".to_owned(),
            language: Language::Rust,
            content: "fn run() {}".to_owned(),
            generated: false,
        };
        let error = build_repository_index(&[source.clone(), source], &[&rust])
            .expect_err("duplicate path must fail");
        assert!(matches!(error, CodeIntelligenceError::DuplicatePath(_)));
    }

    #[test]
    fn missing_driver_fails_closed() {
        let source = SourceFile {
            path: "game.cpp".to_owned(),
            language: Language::Cpp,
            content: "class Player {};".to_owned(),
            generated: false,
        };
        let error = build_repository_index(&[source], &[]).expect_err("missing driver must fail");
        assert!(matches!(
            error,
            CodeIntelligenceError::MissingDriver(Language::Cpp)
        ));
    }

    #[test]
    fn change_impact_reports_dependent_files_deterministically() {
        let rust = LexicalLanguageDriver::rust();
        let sources = vec![
            SourceFile {
                path: "core.rs".to_owned(),
                language: Language::Rust,
                content: "fn core() {}".to_owned(),
                generated: false,
            },
            SourceFile {
                path: "consumer.rs".to_owned(),
                language: Language::Rust,
                content: "use core.rs;\nfn consume() {}".to_owned(),
                generated: false,
            },
        ];
        let index = build_repository_index(&sources, &[&rust]).expect("valid index");
        let impact = change_impact(&index, "core.rs");
        assert_eq!(impact.dependent_paths, vec!["consumer.rs"]);
    }
}

/// Result of deterministic incremental repository indexing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncrementalIndexResult {
    pub index: RepositoryIndex,
    pub reused_files: usize,
    pub reindexed_files: usize,
    pub removed_files: usize,
}

/// Incremental-index validation or prior-evidence failure.
#[derive(Debug)]
pub enum IncrementalIndexError {
    Index(CodeIntelligenceError),
    PriorEvidenceMismatch {
        expected: String,
        actual: String,
    },
}

impl fmt::Display for IncrementalIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index(error) => write!(formatter, "incremental index failed: {error}"),
            Self::PriorEvidenceMismatch { expected, actual } => write!(
                formatter,
                "prior repository evidence mismatch: expected {expected}, recomputed {actual}"
            ),
        }
    }
}

impl Error for IncrementalIndexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Index(error) => Some(error),
            Self::PriorEvidenceMismatch { .. } => None,
        }
    }
}

impl From<CodeIntelligenceError> for IncrementalIndexError {
    fn from(error: CodeIntelligenceError) -> Self {
        Self::Index(error)
    }
}

/// Build a deterministic repository index while reusing exact unchanged prior file evidence.
///
/// Reuse requires the same path, language, generated-file state, and SHA-256 source-content digest.
/// Prior repository evidence is recomputed and verified before any cached file is trusted. Exact
/// language-driver availability remains mandatory even for a reusable file, so cache reuse cannot
/// silently relax the configured language boundary. Removed files are omitted from the new index.
///
/// # Errors
/// Returns [`IncrementalIndexError`] when prior evidence does not verify or normal indexing
/// validation fails.
pub fn build_repository_index_incremental(
    sources: &[SourceFile],
    drivers: &[&dyn LanguageDriver],
    previous: Option<&RepositoryIndex>,
) -> Result<IncrementalIndexResult, IncrementalIndexError> {
    if let Some(previous) = previous {
        let actual = repository_digest(&previous.files)?;
        if actual != previous.evidence_sha256 {
            return Err(IncrementalIndexError::PriorEvidenceMismatch {
                expected: previous.evidence_sha256.clone(),
                actual,
            });
        }
    }

    let mut files = BTreeMap::new();
    let mut reused_files = 0;
    let mut reindexed_files = 0;

    for source in sources {
        if source.path.trim().is_empty() {
            return Err(CodeIntelligenceError::EmptyPath.into());
        }
        if files.contains_key(&source.path) {
            return Err(CodeIntelligenceError::DuplicatePath(source.path.clone()).into());
        }

        let driver = drivers
            .iter()
            .find(|driver| driver.language() == source.language)
            .ok_or(CodeIntelligenceError::MissingDriver(source.language))?;
        let content_sha256 = sha256_hex(source.content.as_bytes());
        let reusable = previous
            .and_then(|index| index.files.get(&source.path))
            .filter(|file| {
                file.path == source.path
                    && file.language == source.language
                    && file.generated == source.generated
                    && file.content_sha256 == content_sha256
            });

        let file_index = if let Some(file) = reusable {
            reused_files += 1;
            file.clone()
        } else {
            reindexed_files += 1;
            driver.index(source)?
        };
        files.insert(source.path.clone(), file_index);
    }

    let removed_files = previous.map_or(0, |index| {
        index
            .files
            .keys()
            .filter(|path| !files.contains_key(*path))
            .count()
    });
    let evidence_sha256 = repository_digest(&files)?;

    Ok(IncrementalIndexResult {
        index: RepositoryIndex {
            files,
            evidence_sha256,
        },
        reused_files,
        reindexed_files,
        removed_files,
    })
}

#[cfg(test)]
mod incremental_tests {
    use super::*;

    struct ReuseOnlyRustDriver;

    impl LanguageDriver for ReuseOnlyRustDriver {
        fn language(&self) -> Language {
            Language::Rust
        }

        fn index(&self, _source: &SourceFile) -> Result<FileIndex, CodeIntelligenceError> {
            panic!("unchanged cached source must not be reindexed")
        }
    }

    fn rust_source(path: &str, content: &str, generated: bool) -> SourceFile {
        SourceFile {
            path: path.to_owned(),
            language: Language::Rust,
            content: content.to_owned(),
            generated,
        }
    }

    #[test]
    fn unchanged_files_reuse_verified_prior_index_without_driver_execution() {
        let lexical = LexicalLanguageDriver::rust();
        let sources = vec![rust_source("src/lib.rs", "fn run() {}", false)];
        let previous = build_repository_index(&sources, &[&lexical]).expect("initial index");
        let reuse_only = ReuseOnlyRustDriver;

        let result = build_repository_index_incremental(&sources, &[&reuse_only], Some(&previous))
            .expect("verified unchanged index should be reusable");

        assert_eq!(result.index, previous);
        assert_eq!(result.reused_files, 1);
        assert_eq!(result.reindexed_files, 0);
        assert_eq!(result.removed_files, 0);
    }

    #[test]
    fn changed_content_is_reindexed_and_matches_clean_rebuild() {
        let lexical = LexicalLanguageDriver::rust();
        let before = vec![
            rust_source("src/lib.rs", "fn run() {}", false),
            rust_source("src/keep.rs", "fn keep() {}", false),
        ];
        let previous = build_repository_index(&before, &[&lexical]).expect("initial index");
        let after = vec![
            rust_source("src/lib.rs", "fn run_changed() {}", false),
            rust_source("src/keep.rs", "fn keep() {}", false),
        ];

        let incremental = build_repository_index_incremental(&after, &[&lexical], Some(&previous))
            .expect("incremental index");
        let clean = build_repository_index(&after, &[&lexical]).expect("clean rebuild");

        assert_eq!(incremental.index, clean);
        assert_eq!(incremental.reused_files, 1);
        assert_eq!(incremental.reindexed_files, 1);
        assert_eq!(incremental.removed_files, 0);
    }

    #[test]
    fn generated_state_change_invalidates_cached_file() {
        let lexical = LexicalLanguageDriver::rust();
        let before = vec![rust_source("src/generated.rs", "fn generated() {}", false)];
        let previous = build_repository_index(&before, &[&lexical]).expect("initial index");
        let after = vec![rust_source("src/generated.rs", "fn generated() {}", true)];

        let result = build_repository_index_incremental(&after, &[&lexical], Some(&previous))
            .expect("generated-state change should reindex");

        assert_eq!(result.reused_files, 0);
        assert_eq!(result.reindexed_files, 1);
        assert!(result.index.files["src/generated.rs"].generated);
    }

    #[test]
    fn removed_files_are_reported_and_excluded() {
        let lexical = LexicalLanguageDriver::rust();
        let before = vec![
            rust_source("src/lib.rs", "fn run() {}", false),
            rust_source("src/removed.rs", "fn removed() {}", false),
        ];
        let previous = build_repository_index(&before, &[&lexical]).expect("initial index");
        let after = vec![rust_source("src/lib.rs", "fn run() {}", false)];

        let result = build_repository_index_incremental(&after, &[&lexical], Some(&previous))
            .expect("removed file should be omitted");

        assert_eq!(result.reused_files, 1);
        assert_eq!(result.reindexed_files, 0);
        assert_eq!(result.removed_files, 1);
        assert!(!result.index.files.contains_key("src/removed.rs"));
    }

    #[test]
    fn tampered_prior_evidence_fails_closed_before_cache_reuse() {
        let lexical = LexicalLanguageDriver::rust();
        let sources = vec![rust_source("src/lib.rs", "fn run() {}", false)];
        let mut previous = build_repository_index(&sources, &[&lexical]).expect("initial index");
        previous.evidence_sha256 = "tampered".to_owned();

        let error = build_repository_index_incremental(&sources, &[&lexical], Some(&previous))
            .expect_err("tampered prior evidence must fail closed");

        assert!(matches!(
            error,
            IncrementalIndexError::PriorEvidenceMismatch { .. }
        ));
    }

    #[test]
    fn equivalent_source_order_produces_identical_incremental_index() {
        let lexical = LexicalLanguageDriver::rust();
        let first = vec![
            rust_source("src/a.rs", "fn a() {}", false),
            rust_source("src/b.rs", "fn b() {}", false),
        ];
        let previous = build_repository_index(&first, &[&lexical]).expect("initial index");
        let mut second = first.clone();
        second.reverse();

        let first_result = build_repository_index_incremental(&first, &[&lexical], Some(&previous))
            .expect("first incremental index");
        let second_result =
            build_repository_index_incremental(&second, &[&lexical], Some(&previous))
                .expect("second incremental index");

        assert_eq!(first_result, second_result);
    }
}
