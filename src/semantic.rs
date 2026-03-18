use crate::config::Config;
use crate::embed::EmbedClient;
use crate::store::{Memory, Store};
use crate::vectordb::VectorDb;
use rig::embeddings::Embedding;
use rig::embeddings::distance::VectorDistance;

pub const MIN_SEMANTIC_SCORE: f64 = 0.5;

pub async fn semantic_search(query: &str, limit: usize) -> anyhow::Result<Vec<Memory>> {
    let config = Config::load()?;
    let store = Store::from_config();
    let vector_db = VectorDb::open().await?;

    let rows = vector_db.load_all().await?;
    if rows.is_empty() {
        return Ok(vec![]);
    }

    let embed_client = EmbedClient::from_config(&config)?;
    let query_vec = embed_client.embed_query(query).await?;
    let query_embedding = Embedding { document: String::new(), vec: query_vec };

    let mut scored: Vec<(String, f64)> = rows
        .into_iter()
        .filter_map(|(id, vec)| {
            let stored = Embedding { document: String::new(), vec };
            let score = query_embedding.cosine_similarity(&stored, false);
            if score >= MIN_SEMANTIC_SCORE {
                Some((id, score))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    let results: Vec<Memory> = scored
        .iter()
        .filter_map(|(id, _)| match store.get(id) {
            Ok(Some(m)) => Some(m),
            Ok(None) => {
                eprintln!(
                    "Warning: memory '{}' found in vector index but not in store (run `ikkinchi reindex`)",
                    id
                );
                None
            }
            Err(e) => {
                eprintln!("Warning: failed to load memory '{}': {}", id, e);
                None
            }
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration test: requires indexed vectors AND Ollama stopped.
    // Run manually: cargo test -- --ignored test_semantic_search_returns_err_when_ollama_down
    #[tokio::test]
    #[ignore]
    async fn test_semantic_search_returns_err_when_ollama_down() {
        let result = semantic_search("rust ownership", 10).await;
        assert!(result.is_err());
    }
}
