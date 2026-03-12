use crate::config::Config;
use crate::embed::EmbedClient;
use crate::store::Store;
use crate::vectordb::VectorDb;
use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rig::embeddings::Embedding;
use rig::embeddings::distance::VectorDistance;
use std::collections::HashMap;

/// Merges semantic (id, score) and fuzzy (id, score) results.
/// Normalizes both to [0,1], computes 0.6*semantic + 0.4*fuzzy, deduplicates.
pub fn hybrid_rank(
    semantic: &[(String, f64)],  // (id, cosine_score)
    fuzzy: &[(String, f64)],     // (id, fuzzy_score) — scores already >= 0
    limit: usize,
) -> Vec<(String, f64)> {
    // Find max semantic score; if 0.0 use 1.0 to avoid div-by-zero.
    let max_semantic = semantic.iter().map(|(_, s)| *s).fold(0.0_f64, f64::max);
    let max_semantic = if max_semantic == 0.0 { 1.0 } else { max_semantic };

    // Find max fuzzy score; if 0.0 use 1.0.
    let max_fuzzy = fuzzy.iter().map(|(_, s)| *s).fold(0.0_f64, f64::max);
    let max_fuzzy = if max_fuzzy == 0.0 { 1.0 } else { max_fuzzy };

    // Build lookup maps
    let semantic_map: HashMap<&str, f64> = semantic
        .iter()
        .map(|(id, s)| (id.as_str(), s / max_semantic))
        .collect();
    let fuzzy_map: HashMap<&str, f64> = fuzzy
        .iter()
        .map(|(id, s)| (id.as_str(), s / max_fuzzy))
        .collect();

    // Collect all unique IDs from both lists
    let mut all_ids: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (id, _) in semantic.iter().chain(fuzzy.iter()) {
        if seen.insert(id.as_str()) {
            all_ids.push(id.clone());
        }
    }

    // For each ID: combined = 0.6 * normalized_semantic + 0.4 * normalized_fuzzy
    let mut scored: Vec<(String, f64)> = all_ids
        .into_iter()
        .map(|id| {
            let s = semantic_map.get(id.as_str()).copied().unwrap_or(0.0);
            let f = fuzzy_map.get(id.as_str()).copied().unwrap_or(0.0);
            let combined = 0.6 * s + 0.4 * f;
            (id, combined)
        })
        .collect();

    // Sort by combined score descending
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Truncate to limit
    scored.truncate(limit);

    scored
}

/// Minimum cosine similarity score for a result to be included in semantic search.
/// Filters out entries that are semantically unrelated to the query.
pub const MIN_SEMANTIC_SCORE: f64 = 0.5;

/// Perform semantic search using vectors loaded from SQLite.
async fn semantic_search(
    embed_client: &EmbedClient,
    vector_db: &VectorDb,
    query: &str,
    limit: usize,
) -> Result<Vec<(String, f64)>> {
    let rows = vector_db.load_all().await?;
    if rows.is_empty() {
        return Ok(vec![]);
    }

    let query_vec = Embedding {
        document: String::new(),
        vec: embed_client.embed_query(query).await?,
    };

    let mut results: Vec<(String, f64)> = rows
        .into_iter()
        .filter_map(|(id, vec)| {
            let stored = Embedding { document: String::new(), vec };
            let score = query_vec.cosine_similarity(&stored, false);
            if score >= MIN_SEMANTIC_SCORE { Some((id, score)) } else { None }
        })
        .collect();

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);

    Ok(results)
}


pub async fn run(query: &str, tag: Option<String>) -> Result<()> {
    let config = Config::load()?;
    let limit = config.display.list_count;
    let store = Store::from_config();

    // 1. Try semantic search
    let semantic_results: Vec<(String, f64)> =
        match (EmbedClient::from_config(&config), VectorDb::open().await) {
            (Ok(embed_client), Ok(vector_db)) => {
                match semantic_search(&embed_client, &vector_db, query, limit).await {
                    Ok(results) => results,
                    Err(e) => {
                        eprintln!("Warning: semantic search failed, falling back to fuzzy only: {}", e);
                        vec![]
                    }
                }
            }
            (Err(e), _) => {
                eprintln!("Warning: failed to init embed client, falling back to fuzzy only: {}", e);
                vec![]
            }
            (_, Err(e)) => {
                eprintln!("Warning: failed to open vector db, falling back to fuzzy only: {}", e);
                vec![]
            }
        };

    // 2. Fuzzy search — run inline to get (id, score) pairs.
    // Only keep results within 50% of the top fuzzy score to filter out weak
    // subsequence matches (e.g. "w-o-r-k" spread across unrelated sentences).
    let matcher = SkimMatcherV2::default();
    let all_memories = store.list(usize::MAX)?;
    let fuzzy_results: Vec<(String, f64)> = {
        let mut scored: Vec<(String, f64)> = all_memories
            .iter()
            .filter_map(|m| {
                matcher
                    .fuzzy_match(&m.text, query)
                    .map(|score| (m.id.clone(), score as f64))
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(&(_, max_score)) = scored.first() {
            let threshold = max_score * 0.5;
            scored.retain(|(_, s)| *s >= threshold);
        }
        scored.truncate(limit);
        scored
    };

    // 3. Hybrid rank
    let ranked = hybrid_rank(&semantic_results, &fuzzy_results, limit);

    if ranked.is_empty() {
        match &tag {
            None => println!("No memories found for: {}", query),
            Some(t) => println!("No results for: {} (tag: #{})", query, t.to_lowercase()),
        }
        return Ok(());
    }

    // 4. Resolve IDs back to Memory structs and apply optional tag filter
    let mut display_index = 1;
    let tag_normalized = tag.as_deref().map(|t| t.to_lowercase());
    for (id, _score) in &ranked {
        match store.get(id)? {
            Some(m) => {
                if let Some(ref t) = tag_normalized {
                    if !m.tags.contains(t) {
                        continue;
                    }
                }
                println!("{:>3}  {}  {}", display_index, m.id, m.text);
                display_index += 1;
            }
            None => {
                eprintln!("Warning: memory '{}' found in vector index but not in store (run `ikkinchi reindex`)", id);
            }
        }
    }

    if display_index == 1 {
        if let Some(t) = &tag {
            println!("No results for: {} (tag: #{})", query, t.to_lowercase());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_rank_semantic_only() {
        let semantic = vec![("a".to_string(), 0.9), ("b".to_string(), 0.5)];
        let fuzzy = vec![];
        let result = hybrid_rank(&semantic, &fuzzy, 10);
        assert_eq!(result[0].0, "a");
        assert_eq!(result[1].0, "b");
    }

    #[test]
    fn test_hybrid_rank_fuzzy_only() {
        let semantic = vec![];
        let fuzzy = vec![("x".to_string(), 100.0), ("y".to_string(), 50.0)];
        let result = hybrid_rank(&semantic, &fuzzy, 10);
        assert_eq!(result[0].0, "x");
        assert_eq!(result[1].0, "y");
    }

    #[test]
    fn test_hybrid_rank_merges_and_deduplicates() {
        let semantic = vec![("a".to_string(), 1.0), ("b".to_string(), 0.5)];
        let fuzzy = vec![("b".to_string(), 100.0), ("c".to_string(), 50.0)];
        let result = hybrid_rank(&semantic, &fuzzy, 10);
        // "b" appears in both — should appear once
        let ids: Vec<&str> = result.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids.iter().filter(|&&id| id == "b").count(), 1);
        // All 3 unique IDs should appear
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_hybrid_rank_respects_limit() {
        let semantic = vec![("a".to_string(), 1.0), ("b".to_string(), 0.8), ("c".to_string(), 0.6)];
        let fuzzy = vec![];
        let result = hybrid_rank(&semantic, &fuzzy, 2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_hybrid_rank_empty_inputs() {
        let result = hybrid_rank(&[], &[], 10);
        assert!(result.is_empty());
    }
}
