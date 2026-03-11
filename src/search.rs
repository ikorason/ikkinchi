use crate::store::{Memory, Store};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub fn fuzzy_search(store: &Store, query: &str, limit: usize) -> anyhow::Result<Vec<Memory>> {
    let matcher = SkimMatcherV2::default();
    let all = store.list(usize::MAX)?;

    let mut scored: Vec<(i64, Memory)> = all
        .into_iter()
        .filter_map(|m| matcher.fuzzy_match(&m.text, query).map(|score| (score, m)))
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.truncate(limit);

    Ok(scored.into_iter().map(|(_, m)| m).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn test_store() -> (TempDir, Store) {
        let dir = TempDir::new().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        (dir, store)
    }

    #[test]
    fn test_fuzzy_search_returns_matching_memory() {
        let (_dir, store) = test_store();
        fs::create_dir_all(&store.memories_dir).unwrap();
        fs::write(
            store.memories_dir.join("2026-03-10.md"),
            "## 10:00\n\nevent sourcing idea\n\n## 11:00\n\nrust ownership model\n\n",
        )
        .unwrap();

        let results = fuzzy_search(&store, "event", 20).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "event sourcing idea");
    }

    #[test]
    fn test_fuzzy_search_no_match_returns_empty() {
        let (_dir, store) = test_store();
        fs::create_dir_all(&store.memories_dir).unwrap();
        fs::write(
            store.memories_dir.join("2026-03-10.md"),
            "## 10:00\n\nhello world\n\n",
        )
        .unwrap();

        let results = fuzzy_search(&store, "zzzzz", 20).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_orders_by_score_descending() {
        let (_dir, store) = test_store();
        fs::create_dir_all(&store.memories_dir).unwrap();
        fs::write(
            store.memories_dir.join("2026-03-10.md"),
            "## 10:00\n\nrust ownership model\n\n## 11:00\n\nlearning a systems language\n\n",
        )
        .unwrap();

        let results = fuzzy_search(&store, "rust", 20).unwrap();
        assert_eq!(results[0].text, "rust ownership model");
    }

    #[test]
    fn test_fuzzy_search_respects_limit() {
        let (_dir, store) = test_store();
        fs::create_dir_all(&store.memories_dir).unwrap();
        fs::write(
            store.memories_dir.join("2026-03-10.md"),
            "## 09:00:00\n\nrust error handling\n\n## 10:00:00\n\nrust ownership\n\n## 11:00:00\n\nrust lifetimes\n\n",
        )
        .unwrap();

        let results = fuzzy_search(&store, "rust", 2).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_fuzzy_search_empty_store_returns_empty() {
        let (_dir, store) = test_store();
        let results = fuzzy_search(&store, "anything", 20).unwrap();
        assert!(results.is_empty());
    }
}
