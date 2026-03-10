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

    pub fn append(&self, text: &str) -> anyhow::Result<String> {
        let now = chrono::Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        let time = now.format("%H:%M").to_string();

        std::fs::create_dir_all(&self.memories_dir)?;
        let file_path = self.memories_dir.join(format!("{}.md", date));

        let entry = format!("## {}\n\n{}\n\n", time, text);
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        file.write_all(entry.as_bytes())?;

        Ok(format!("{}/{}", date, time))
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

    #[test]
    fn test_append_creates_file_with_correct_format() {
        let (_dir, store) = test_store();
        let id = store.append("hello world").unwrap();

        // id must be "YYYY-MM-DD/HH:MM"
        let parts: Vec<&str> = id.splitn(2, '/').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].len(), 5); // "HH:MM"

        // file must exist
        let file_path = store.memories_dir.join(format!("{}.md", parts[0]));
        assert!(file_path.exists());

        // file must contain the entry
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("hello world"));
        assert!(content.contains(&format!("## {}", parts[1])));
    }

    #[test]
    fn test_append_twice_same_file_keeps_both() {
        let (_dir, store) = test_store();
        store.append("first").unwrap();
        store.append("second").unwrap();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let file_path = store.memories_dir.join(format!("{}.md", today));
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("first"));
        assert!(content.contains("second"));
    }
}
