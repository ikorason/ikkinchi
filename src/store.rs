use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Memory {
    pub id: String,   // "2026-03-10/14:32"
    pub date: String, // "2026-03-10"
    pub time: String, // "14:32"
    pub text: String,
}

impl Memory {
    pub fn new(date: &str, time: &str, text: &str) -> Self {
        Self {
            id: format!("{}/{}", date, time),
            date: date.to_string(),
            time: time.to_string(),
            text: text.to_string(),
        }
    }
}

pub struct Store {
    pub memories_dir: PathBuf,
}

impl Store {
    pub fn new(memories_dir: PathBuf) -> Self {
        Self { memories_dir }
    }

    pub fn from_config() -> Self {
        Self::new(crate::config::memories_dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_store() -> (TempDir, Store) {
        let dir = TempDir::new().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        (dir, store)
    }

    #[test]
    fn test_memory_new() {
        let m = Memory::new("2026-03-10", "14:32", "hello");
        assert_eq!(m.id, "2026-03-10/14:32");
        assert_eq!(m.date, "2026-03-10");
        assert_eq!(m.time, "14:32");
        assert_eq!(m.text, "hello");
    }

    #[test]
    fn test_store_new() {
        let (_dir, store) = test_store();
        assert!(store.memories_dir.to_str().is_some());
    }
}
