use crate::config::Config;
use crate::store::{Memory, Store};
use anyhow::Result;

pub fn format_tag_block(m: &Memory) -> String {
    if m.tags.is_empty() {
        return String::new();
    }
    let tags = m.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(", ");
    format!("[{}] ", tags)
}

pub fn filter_by_tag<'a>(memories: &'a [Memory], tag: &str) -> Vec<&'a Memory> {
    let normalized = tag.to_lowercase();
    memories.iter().filter(|m| m.tags.contains(&normalized)).collect()
}

pub async fn run(count: Option<usize>, tag: Option<String>) -> Result<()> {
    let config = Config::load()?;
    let limit = count.unwrap_or(config.display.list_count);
    let store = Store::from_config();
    let memories = store.list(limit)?;

    if memories.is_empty() {
        println!("No memories yet. Try: ikkinchi add \"your thought\"");
        return Ok(());
    }

    match &tag {
        None => {
            for (i, m) in memories.iter().enumerate() {
                println!("{:>3}  {}  {}{}", i + 1, m.id, format_tag_block(m), m.text);
            }
        }
        Some(t) => {
            let filtered = filter_by_tag(&memories, t);
            if filtered.is_empty() {
                println!("No memories with tag: #{}", t.to_lowercase());
            } else {
                for (i, m) in filtered.iter().enumerate() {
                    println!("{:>3}  {}  {}{}", i + 1, m.id, format_tag_block(m), m.text);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Memory;

    fn make_memory(date: &str, time: &str, text: &str, tags: Vec<&str>) -> Memory {
        let mut m = Memory::new(date, time, text);
        m.tags = tags.into_iter().map(|s| s.to_string()).collect();
        m
    }

    #[test]
    fn test_format_tag_block_empty() {
        let m = make_memory("2026-03-11", "10:00:00", "text", vec![]);
        assert_eq!(format_tag_block(&m), "");
    }

    #[test]
    fn test_format_tag_block_single() {
        let m = make_memory("2026-03-11", "10:00:00", "text", vec!["rust"]);
        assert_eq!(format_tag_block(&m), "[#rust] ");
    }

    #[test]
    fn test_format_tag_block_multiple() {
        let m = make_memory("2026-03-11", "10:00:00", "text", vec!["rust", "til"]);
        assert_eq!(format_tag_block(&m), "[#rust, #til] ");
    }

    #[test]
    fn test_filter_by_tag_returns_matches() {
        let memories = vec![
            make_memory("2026-03-11", "10:00:00", "first", vec!["rust"]),
            make_memory("2026-03-11", "11:00:00", "second", vec!["til"]),
        ];
        let result = filter_by_tag(&memories, "rust");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "first");
    }

    #[test]
    fn test_filter_by_tag_case_insensitive() {
        let memories = vec![
            make_memory("2026-03-11", "10:00:00", "first", vec!["rust"]),
        ];
        let result = filter_by_tag(&memories, "RUST");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_by_tag_no_match_returns_empty() {
        let memories = vec![
            make_memory("2026-03-11", "10:00:00", "first", vec!["rust"]),
        ];
        let result = filter_by_tag(&memories, "haskell");
        assert!(result.is_empty());
    }
}
