use crate::store::{Memory, Store};
use anyhow::Result;
use std::collections::HashMap;

pub fn count_tags(memories: &[Memory]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for m in memories {
        for tag in &m.tags {
            *counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    sorted
}

pub async fn run() -> Result<()> {
    let store = Store::from_config();
    let memories = store.list(usize::MAX)?;
    let counts = count_tags(&memories);
    if counts.is_empty() {
        println!("No tags yet.");
        return Ok(());
    }
    let max_tag_len = counts.iter().map(|(t, _)| t.len() + 1).max().unwrap_or(0);
    for (tag, count) in &counts {
        let display = format!("#{}", tag);
        println!("  {:<width$}  {}", display, count, width = max_tag_len);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Memory;

    fn make_memory(tags: Vec<&str>) -> Memory {
        let mut m = Memory::new("2026-03-11", "10:00:00", "text");
        m.tags = tags.into_iter().map(|s| s.to_string()).collect();
        m
    }

    #[test]
    fn test_count_tags_empty() {
        let result = count_tags(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_count_tags_single() {
        let memories = vec![make_memory(vec!["rust"])];
        let result = count_tags(&memories);
        assert_eq!(result, vec![("rust".to_string(), 1)]);
    }

    #[test]
    fn test_count_tags_multiple_memories() {
        let memories = vec![
            make_memory(vec!["rust", "til"]),
            make_memory(vec!["rust"]),
        ];
        let result = count_tags(&memories);
        let rust = result.iter().find(|(t, _)| t == "rust").unwrap();
        let til = result.iter().find(|(t, _)| t == "til").unwrap();
        assert_eq!(rust.1, 2);
        assert_eq!(til.1, 1);
        assert_eq!(result[0].0, "rust");
    }

    #[test]
    fn test_count_tags_no_tags_returns_empty() {
        let memories = vec![make_memory(vec![])];
        let result = count_tags(&memories);
        assert!(result.is_empty());
    }
}
