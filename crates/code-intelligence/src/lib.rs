//! Provider-neutral, engine-neutral code intelligence substrate for NTPX GameForge Ω∞.
//!
//! Phase 06 starts with deterministic repository indexing, language-driver contracts, symbol and
//! dependency metadata, diagnostics, impact analysis, patch planning, and reproducible evidence.

use std::{collections::BTreeMap, error::Error, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Supported code/document language classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Rust,
    TypeScript,
    Cpp,
    BlueprintMetadata,
    Unknown,
}

/// Repository source input used by language drivers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub language: Language,
    pub content: String,
    pub generated: bool,
}

/// Stable symbol record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub id: String,
    pub path: String,
    pub name: String,
    pub line: u32,
}

/// Stable dependency record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyRecord {
    pub from_path: String,
    pub target: String,
}

/// Driver-produced file index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileIndex {
    pub path: String,
    pub language: Language,
    pub symbols: Vec<SymbolRecord>,
    pub dependencies: Vec<DependencyRecord>,
    pub content_sha256: String,
    pub generated: bool,
}

/// Repository-level deterministic index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryIndex {
    pub files: BTreeMap<String, FileIndex>,
}

/// Change-impact output. This is advisory evidence, not authority to modify code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeImpact {
    pub changed_path: String,
    pub dependent_paths: Vec<String>,
}

/// Explicit patch-planning record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchPlan {
    pub objective: String,
    pub paths: Vec<String>,
    pub requires_generated_file_edit: bool,
}

/// Language indexing contract.
pub trait LanguageDriver: Send + Sync {
    fn language(&self) -> Language;
    fn index(&self, source: &SourceFile) -> Result<FileIndex, CodeIntelligenceError>;
}

/// Lightweight deterministic lexical driver used until AST-native drivers are introduced.
#[derive(Debug, Clone, Copy)]
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

    fn symbol_prefixes(self) -> &'static [&'static str] {
        match self.language {
            Language::Rust => &["fn ", "struct ", "enum ", "trait ", "mod "],
            Language::TypeScript => &["function ", "class ", "interface ", "type ", "const "],
            Language::Cpp => &["class ", "struct ", "enum ", "void ", "int ", "bool "],
            Language::BlueprintMetadata => &["Function=", "Class=", "Variable="],
            Language::Unknown => &[],
        }
    }

    fn dependency_prefixes(self) -> &'static [&'static str] {
        match self.language {
            Language::Rust => &["use ", "mod "],
            Language::TypeScript => &["import ", "export "],
            Language::Cpp => &["#include "],
            Language::BlueprintMetadata => &["ParentClass=", "Asset="],
            Language::Unknown => &[],
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
        let mut dependencies = Vec::new();
        for (index, raw_line) in source.content.lines().enumerate() {
            let line = raw_line.trim();
            let line_number = u32::try_from(index + 1).unwrap_or(u32::MAX);
            if let Some(name) = extract_prefixed_name(line, self.symbol_prefixes()) {
                symbols.push(SymbolRecord {
                    id: stable_symbol_id(&source.path, &name, line_number),
                    path: source.path.clone(),
                    name,
                    line: line_number,
                });
            }
            if let Some(target) = extract_prefixed_name(line, self.dependency_prefixes()) {
                dependencies.push(DependencyRecord {
                    from_path: source.path.clone(),
                    target,
                });
            }
        }
        symbols.sort_by(|left, right| left.id.cmp(&right.id));
        dependencies.sort_by(|left, right| left.target.cmp(&right.target));
        Ok(FileIndex {
            path: source.path.clone(),
            language: source.language,
            symbols,
            dependencies,
            content_sha256: sha256_hex(source.content.as_bytes()),
            generated: source.generated,
        })
    }
}

fn extract_prefixed_name(line: &str, prefixes: &[&str]) -> Option<String> {
    prefixes.iter().find_map(|prefix| {
        line.strip_prefix(prefix).and_then(|rest| {
            let token = rest
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
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

/// Code intelligence failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeIntelligenceError {
    EmptyPath,
    DuplicatePath(String),
    MissingDriver(Language),
    LanguageMismatch {
        path: String,
        expected: Language,
        actual: Language,
    },
    EmptyPatchObjective,
    EmptyPatchPaths,
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
            Self::EmptyPatchObjective => formatter.write_str("patch objective must not be empty"),
            Self::EmptyPatchPaths => formatter.write_str("patch plan must include at least one path"),
        }
    }
}

impl Error for CodeIntelligenceError {}

/// Build a deterministic repository index with exact language-driver matching.
///
/// # Errors
/// Returns [`CodeIntelligenceError`] for duplicate paths, empty paths, or missing/mismatched drivers.
pub fn build_repository_index(
    sources: &[SourceFile],
    drivers: &[&dyn LanguageDriver],
) -> Result<RepositoryIndex, CodeIntelligenceError> {
    let mut files = BTreeMap::new();
    for source in sources {
        let driver = drivers
            .iter()
            .find(|driver| driver.language() == source.language)
            .ok_or(CodeIntelligenceError::MissingDriver(source.language))?;
        let index = driver.index(source)?;
        if files.insert(source.path.clone(), index).is_some() {
            return Err(CodeIntelligenceError::DuplicatePath(source.path.clone()));
        }
    }
    Ok(RepositoryIndex { files })
}

/// Determine which indexed files lexically depend on one changed path.
#[must_use]
pub fn change_impact(index: &RepositoryIndex, changed_path: &str) -> ChangeImpact {
    let mut dependent_paths: Vec<_> = index
        .files
        .values()
        .filter(|file| {
            file.dependencies
                .iter()
                .any(|dependency| dependency.target.contains(changed_path))
        })
        .map(|file| file.path.clone())
        .collect();
    dependent_paths.sort();
    ChangeImpact {
        changed_path: changed_path.to_owned(),
        dependent_paths,
    }
}

/// Construct a patch plan while surfacing generated-file edits explicitly.
///
/// # Errors
/// Returns [`CodeIntelligenceError`] for an empty objective or empty target path set.
pub fn plan_patch(
    index: &RepositoryIndex,
    objective: impl Into<String>,
    mut paths: Vec<String>,
) -> Result<PatchPlan, CodeIntelligenceError> {
    let objective = objective.into();
    if objective.trim().is_empty() {
        return Err(CodeIntelligenceError::EmptyPatchObjective);
    }
    if paths.is_empty() {
        return Err(CodeIntelligenceError::EmptyPatchPaths);
    }
    paths.sort();
    paths.dedup();
    let requires_generated_file_edit = paths
        .iter()
        .filter_map(|path| index.files.get(path))
        .any(|file| file.generated);
    Ok(PatchPlan {
        objective,
        paths,
        requires_generated_file_edit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_index_is_deterministic() {
        let rust = LexicalLanguageDriver::rust();
        let ts = LexicalLanguageDriver::typescript();
        let sources = vec![
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
        ];
        let index = build_repository_index(&sources, &[&rust, &ts]).expect("valid index");
        assert_eq!(index.files.len(), 2);
        assert_eq!(index.files["src/lib.rs"].symbols[0].name, "run");
    }

    #[test]
    fn generated_file_edits_are_explicit() {
        let rust = LexicalLanguageDriver::rust();
        let sources = vec![SourceFile {
            path: "generated.rs".to_owned(),
            language: Language::Rust,
            content: "struct Generated;".to_owned(),
            generated: true,
        }];
        let index = build_repository_index(&sources, &[&rust]).expect("valid index");
        let plan = plan_patch(&index, "update generated output", vec!["generated.rs".to_owned()])
            .expect("valid patch plan");
        assert!(plan.requires_generated_file_edit);
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
        assert_eq!(error, CodeIntelligenceError::MissingDriver(Language::Cpp));
    }
}
