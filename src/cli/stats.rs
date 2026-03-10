use crate::store::Store;
use anyhow::Result;
use std::collections::HashSet;

pub struct Stats {
    pub total: usize,
    pub days: usize,
    pub oldest: String,
    pub newest: String,
}

pub fn compute_stats(store: &Store) -> Result<Option<Stats>> {
    let all = store.list(usize::MAX)?;
    if all.is_empty() {
        return Ok(None);
    }
    let days: HashSet<&str> = all.iter().map(|m| m.date.as_str()).collect();
    let oldest = all.iter().map(|m| m.date.as_str()).min().unwrap().to_string();
    let newest = all.iter().map(|m| m.date.as_str()).max().unwrap().to_string();
    Ok(Some(Stats {
        total: all.len(),
        days: days.len(),
        oldest,
        newest,
    }))
}

pub async fn run() -> Result<()> {
    let store = Store::from_config();
    match compute_stats(&store)? {
        None => println!("No memories yet."),
        Some(s) => {
            println!("Memories: {:>10}", s.total);
            println!("Days:     {:>10}", s.days);
            println!("Oldest:   {:>10}", s.oldest);
            println!("Newest:   {:>10}", s.newest);
        }
    }
    Ok(())
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
    fn test_stats_empty_store_returns_none() {
        let (_dir, store) = test_store();
        assert!(compute_stats(&store).unwrap().is_none());
    }

    #[test]
    fn test_stats_single_day() {
        let (_dir, store) = test_store();
        fs::create_dir_all(&store.memories_dir).unwrap();
        fs::write(
            store.memories_dir.join("2026-03-10.md"),
            "## 10:00\n\nhello\n\n## 11:00\n\nworld\n\n",
        ).unwrap();

        let stats = compute_stats(&store).unwrap().unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.days, 1);
        assert_eq!(stats.oldest, "2026-03-10");
        assert_eq!(stats.newest, "2026-03-10");
    }

    #[test]
    fn test_stats_multiple_days() {
        let (_dir, store) = test_store();
        fs::create_dir_all(&store.memories_dir).unwrap();
        fs::write(
            store.memories_dir.join("2026-03-09.md"),
            "## 10:00\n\nold thought\n\n",
        ).unwrap();
        fs::write(
            store.memories_dir.join("2026-03-10.md"),
            "## 10:00\n\nnew thought\n\n## 11:00\n\nanother\n\n",
        ).unwrap();

        let stats = compute_stats(&store).unwrap().unwrap();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.days, 2);
        assert_eq!(stats.oldest, "2026-03-09");
        assert_eq!(stats.newest, "2026-03-10");
    }
}
