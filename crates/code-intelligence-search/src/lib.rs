//! Deterministic advisory symbol search over verified Code Intelligence index evidence.
//!
//! This crate performs local lexical ranking only. It does not provide embeddings, compiler/LSP
//! semantics, source mutation, model invocation, tool execution, network access, or authorization.

use std::{error::Error, fmt};

use code_intelligence::{RepositoryIndex, SymbolRecord};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// One deterministic symbol-search hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolSearchHit {
    pub symbol: SymbolRecord,
    pub score: u16,
}

/// Deterministic advisory symbol-search result and correlation evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolSearchResult {
    pub normalized_query: String,
    pub hits: Vec<SymbolSearchHit>,
    pub evidence_sha256: String,
}

/// Symbol-search validation or evidence encoding failure.
#[derive(Debug)]
pub enum SymbolSearchError {
    EmptyQuery,
    Encode(serde_json::Error),
}

impl fmt::Display for SymbolSearchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyQuery => formatter.write_str("symbol-search query must not be empty"),
            Self::Encode(error) => {
                write!(formatter, "symbol-search evidence encoding failed: {error}")
            }
        }
    }
}

impl Error for SymbolSearchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::EmptyQuery => None,
        }
    }
}

#[derive(Serialize)]
struct SearchEvidence<'a> {
    repository_evidence_sha256: &'a str,
    normalized_query: &'a str,
    hits: &'a [SymbolSearchHit],
}

/// Search indexed symbols using deterministic local lexical ranking.
///
/// Ranking is exact-name match, then prefix match, then substring match. Ties are resolved by
/// stable path, name, line, and symbol ID ordering. Results never grant mutation or execution
/// authority and must not be represented as compiler/LSP or embedding-based semantic search.
///
/// # Errors
/// Returns [`SymbolSearchError`] for an empty normalized query or evidence serialization failure.
pub fn search_symbols(
    index: &RepositoryIndex,
    query: &str,
    limit: usize,
) -> Result<SymbolSearchResult, SymbolSearchError> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        return Err(SymbolSearchError::EmptyQuery);
    }

    let mut hits: Vec<_> = index
        .files
        .values()
        .flat_map(|file| file.symbols.iter())
        .filter_map(|symbol| {
            let score = lexical_score(&symbol.name, &normalized_query)?;
            Some(SymbolSearchHit {
                symbol: symbol.clone(),
                score,
            })
        })
        .collect();

    hits.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.symbol.path.cmp(&right.symbol.path))
            .then_with(|| left.symbol.name.cmp(&right.symbol.name))
            .then_with(|| left.symbol.line.cmp(&right.symbol.line))
            .then_with(|| left.symbol.id.cmp(&right.symbol.id))
    });
    hits.truncate(limit);

    let encoded = serde_json::to_vec(&SearchEvidence {
        repository_evidence_sha256: &index.evidence_sha256,
        normalized_query: &normalized_query,
        hits: &hits,
    })
    .map_err(SymbolSearchError::Encode)?;
    let evidence_sha256 = format!("{:x}", Sha256::digest(encoded));

    Ok(SymbolSearchResult {
        normalized_query,
        hits,
        evidence_sha256,
    })
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn lexical_score(name: &str, normalized_query: &str) -> Option<u16> {
    let normalized_name = name.to_lowercase();
    if normalized_name == normalized_query {
        Some(300)
    } else if normalized_name.starts_with(normalized_query) {
        Some(200)
    } else if normalized_name.contains(normalized_query) {
        Some(100)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use code_intelligence::{FileIndex, Language};

    use super::*;

    fn symbol(id: &str, path: &str, name: &str, line: u32) -> SymbolRecord {
        SymbolRecord {
            id: id.to_owned(),
            path: path.to_owned(),
            name: name.to_owned(),
            line,
        }
    }

    fn index(symbols: Vec<SymbolRecord>) -> RepositoryIndex {
        let file = FileIndex {
            path: "src/lib.rs".to_owned(),
            language: Language::Rust,
            content_sha256: "content".to_owned(),
            generated: false,
            symbols,
            references: Vec::new(),
        };
        RepositoryIndex {
            files: BTreeMap::from([(file.path.clone(), file)]),
            evidence_sha256: "repository-evidence".to_owned(),
        }
    }

    #[test]
    fn exact_prefix_and_substring_matches_have_stable_priority() {
        let repository = index(vec![
            symbol("3", "src/z.rs", "PlayerController", 30),
            symbol("1", "src/a.rs", "Player", 10),
            symbol("2", "src/b.rs", "LocalPlayer", 20),
        ]);

        let result = search_symbols(&repository, " player ", 10).expect("valid search");

        assert_eq!(result.normalized_query, "player");
        assert_eq!(result.hits.len(), 3);
        assert_eq!(result.hits[0].symbol.name, "Player");
        assert_eq!(result.hits[0].score, 300);
        assert_eq!(result.hits[1].symbol.name, "PlayerController");
        assert_eq!(result.hits[1].score, 200);
        assert_eq!(result.hits[2].symbol.name, "LocalPlayer");
        assert_eq!(result.hits[2].score, 100);
    }

    #[test]
    fn equivalent_symbol_order_produces_identical_result_evidence() {
        let first_symbols = vec![
            symbol("b", "src/b.rs", "RenderWorld", 20),
            symbol("a", "src/a.rs", "RenderActor", 10),
        ];
        let mut second_symbols = first_symbols.clone();
        second_symbols.reverse();

        let first = search_symbols(&index(first_symbols), "render", 10).expect("first search");
        let second = search_symbols(&index(second_symbols), "render", 10).expect("second search");

        assert_eq!(first, second);
    }

    #[test]
    fn result_limit_is_applied_after_deterministic_ranking() {
        let repository = index(vec![
            symbol("c", "src/c.rs", "TaskGamma", 30),
            symbol("a", "src/a.rs", "TaskAlpha", 10),
            symbol("b", "src/b.rs", "TaskBeta", 20),
        ]);

        let result = search_symbols(&repository, "task", 2).expect("valid search");

        assert_eq!(result.hits.len(), 2);
        assert_eq!(result.hits[0].symbol.path, "src/a.rs");
        assert_eq!(result.hits[1].symbol.path, "src/b.rs");
    }

    #[test]
    fn empty_query_fails_closed() {
        let error = search_symbols(&index(Vec::new()), " \n\t ", 10)
            .expect_err("empty query must fail closed");
        assert!(matches!(error, SymbolSearchError::EmptyQuery));
    }

    #[test]
    fn zero_limit_returns_deterministic_empty_evidence() {
        let repository = index(vec![symbol("a", "src/a.rs", "Player", 10)]);
        let first = search_symbols(&repository, "player", 0).expect("valid search");
        let second = search_symbols(&repository, "PLAYER", 0).expect("equivalent search");

        assert!(first.hits.is_empty());
        assert_eq!(first, second);
    }
}
