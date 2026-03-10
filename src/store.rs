use std::io::Write;
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

        let text = text.trim();
        let entry = format!("## {}\n\n{}\n\n", time, text);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        file.write_all(entry.as_bytes())?;

        // Note: ID is YYYY-MM-DD/HH:MM — two calls within the same minute
        // produce the same ID string. Both entries exist in the file, but
        // get/edit/delete will only find the first. Acceptable for v1.
        Ok(format!("{}/{}", date, time))
    }

    pub fn list(&self, limit: usize) -> anyhow::Result<Vec<Memory>> {
        if !self.memories_dir.exists() {
            return Ok(vec![]);
        }

        let mut files: Vec<PathBuf> = std::fs::read_dir(&self.memories_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
            .collect();

        // Sort by filename (date) descending — newest day first
        files.sort();
        files.reverse();

        let mut all: Vec<Memory> = Vec::new();
        for file_path in &files {
            let date = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let content = std::fs::read_to_string(file_path)?;
            let mut memories = parse_file(&date, &content);
            // Within a day: newest time first
            memories.sort_by(|a, b| b.time.cmp(&a.time));
            all.extend(memories);
            if all.len() >= limit {
                break;
            }
        }

        all.truncate(limit);
        Ok(all)
    }
}

fn parse_time_header(line: &str) -> Option<String> {
    let rest = line.strip_prefix("## ")?;
    // Must be exactly HH:MM
    if rest.len() == 5
        && rest.as_bytes()[2] == b':'
        && rest[..2].bytes().all(|b| b.is_ascii_digit())
        && rest[3..].bytes().all(|b| b.is_ascii_digit())
    {
        Some(rest.to_string())
    } else {
        None
    }
}

fn parse_file(date: &str, content: &str) -> Vec<Memory> {
    let mut memories = Vec::new();
    let mut current_time: Option<String> = None;
    let mut current_body: Vec<&str> = Vec::new();

    for line in content.lines() {
        if let Some(time) = parse_time_header(line) {
            if let Some(ref t) = current_time {
                let text = current_body.join("\n").trim().to_string();
                if !text.is_empty() {
                    memories.push(Memory::new(date, t, &text));
                }
            }
            current_time = Some(time);
            current_body.clear();
        } else {
            current_body.push(line);
        }
    }

    if let Some(ref t) = current_time {
        let text = current_body.join("\n").trim().to_string();
        if !text.is_empty() {
            memories.push(Memory::new(date, t, &text));
        }
    }

    memories
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
        assert_eq!(content, format!("## {}\n\nhello world\n\n", parts[1]));
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

    #[test]
    fn test_list_empty_returns_empty() {
        let (_dir, store) = test_store();
        assert!(store.list(20).unwrap().is_empty());
    }

    #[test]
    fn test_list_returns_memories_newest_first() {
        let (_dir, store) = test_store();
        let date = "2026-03-10";
        let file_path = store.memories_dir.join(format!("{}.md", date));
        std::fs::create_dir_all(&store.memories_dir).unwrap();
        std::fs::write(
            &file_path,
            "## 09:00\n\nearly thought\n\n## 15:30\n\nlate thought\n\n",
        ).unwrap();

        let memories = store.list(20).unwrap();
        assert_eq!(memories.len(), 2);
        assert_eq!(memories[0].time, "15:30"); // newest first
        assert_eq!(memories[1].time, "09:00");
    }

    #[test]
    fn test_list_respects_limit() {
        let (_dir, store) = test_store();
        let date = "2026-03-10";
        let file_path = store.memories_dir.join(format!("{}.md", date));
        std::fs::create_dir_all(&store.memories_dir).unwrap();
        std::fs::write(
            &file_path,
            "## 09:00\n\nfirst\n\n## 10:00\n\nsecond\n\n## 11:00\n\nthird\n\n",
        ).unwrap();

        let memories = store.list(2).unwrap();
        assert_eq!(memories.len(), 2);
    }

    #[test]
    fn test_list_multiple_days_ordered() {
        let (_dir, store) = test_store();
        std::fs::create_dir_all(&store.memories_dir).unwrap();
        std::fs::write(
            store.memories_dir.join("2026-03-09.md"),
            "## 10:00\n\nold\n\n",
        ).unwrap();
        std::fs::write(
            store.memories_dir.join("2026-03-10.md"),
            "## 10:00\n\nnew\n\n",
        ).unwrap();

        let memories = store.list(20).unwrap();
        assert_eq!(memories[0].date, "2026-03-10"); // newer day first
        assert_eq!(memories[1].date, "2026-03-09");
    }
}
