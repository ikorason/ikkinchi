use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Memory {
    pub id: String,   // "2026-03-10/14:32:05"
    pub date: String, // "2026-03-10"
    pub time: String, // "14:32:05"
    pub text: String,
    pub tags: Vec<String>,
}

impl Memory {
    pub fn new(date: &str, time: &str, text: &str) -> Self {
        Self {
            id: format!("{}/{}", date, time),
            date: date.to_string(),
            time: time.to_string(),
            text: text.to_string(),
            tags: vec![],
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
        let time = now.format("%H:%M:%S").to_string();

        std::fs::create_dir_all(&self.memories_dir)?;
        let file_path = self.memories_dir.join(format!("{}.md", date));

        let text = text.trim();
        let entry = format!("## {}\n\n{}\n\n", time, text);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        file.write_all(entry.as_bytes())?;

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

    pub fn get(&self, id: &str) -> anyhow::Result<Option<Memory>> {
        let (date, time) = parse_id(id)?;
        let file_path = self.memories_dir.join(format!("{}.md", date));
        if !file_path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&file_path)?;
        Ok(parse_file(&date, &content).into_iter().find(|m| m.time == time))
    }

    pub fn update(&self, id: &str, text: &str) -> anyhow::Result<()> {
        let (date, time) = parse_id(id)?;
        let file_path = self.memories_dir.join(format!("{}.md", date));
        anyhow::ensure!(file_path.exists(), "Memory not found: {}", id);
        let content = std::fs::read_to_string(&file_path)?;
        let mut memories = parse_file(&date, &content);
        let entry = memories
            .iter_mut()
            .find(|m| m.time == time)
            .ok_or_else(|| anyhow::anyhow!("Memory not found: {}", id))?;
        entry.text = text.trim().to_string();
        write_file(&file_path, &memories)
    }

    pub fn delete(&self, id: &str) -> anyhow::Result<()> {
        let (date, time) = parse_id(id)?;
        let file_path = self.memories_dir.join(format!("{}.md", date));
        anyhow::ensure!(file_path.exists(), "Memory not found: {}", id);
        let content = std::fs::read_to_string(&file_path)?;
        let memories: Vec<Memory> = parse_file(&date, &content)
            .into_iter()
            .filter(|m| m.time != time)
            .collect();
        if memories.is_empty() {
            std::fs::remove_file(&file_path)?;
        } else {
            write_file(&file_path, &memories)?;
        }
        Ok(())
    }
}

fn parse_id(id: &str) -> anyhow::Result<(String, String)> {
    let mut parts = id.splitn(2, '/');
    let date = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid memory id: {}", id))?
        .to_string();
    let time = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid memory id: {}", id))?
        .to_string();
    Ok((date, time))
}

fn write_file(path: &PathBuf, memories: &[Memory]) -> anyhow::Result<()> {
    let mut content = String::new();
    for m in memories {
        content.push_str(&format!("## {}\n\n{}\n\n", m.time, m.text));
    }
    std::fs::write(path, content)?;
    Ok(())
}

pub fn parse_tag_line(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return vec![];
    }
    let mut tags = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for part in trimmed.split(',') {
        let raw = part.trim().strip_prefix('#').unwrap_or("").trim();
        let normalized: String = raw
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
            .collect::<String>()
            .to_lowercase();
        let tag = normalized.trim().to_string();
        if !tag.is_empty() && seen.insert(tag.clone()) {
            tags.push(tag);
        }
    }
    tags
}

fn parse_time_header(line: &str) -> Option<String> {
    let rest = line.strip_prefix("## ")?;
    // Accept HH:MM:SS (new format) or HH:MM (backward compat for existing files)
    let valid = if rest.len() == 8 {
        rest.as_bytes()[2] == b':'
            && rest.as_bytes()[5] == b':'
            && rest[..2].bytes().all(|b| b.is_ascii_digit())
            && rest[3..5].bytes().all(|b| b.is_ascii_digit())
            && rest[6..].bytes().all(|b| b.is_ascii_digit())
    } else if rest.len() == 5 {
        rest.as_bytes()[2] == b':'
            && rest[..2].bytes().all(|b| b.is_ascii_digit())
            && rest[3..].bytes().all(|b| b.is_ascii_digit())
    } else {
        false
    };
    if valid { Some(rest.to_string()) } else { None }
}

fn parse_file(date: &str, content: &str) -> Vec<Memory> {
    let mut memories = Vec::new();
    let mut current_time: Option<String> = None;
    let mut current_body: Vec<&str> = Vec::new();

    for line in content.lines() {
        if let Some(time) = parse_time_header(line) {
            if let Some(ref t) = current_time {
                let memory = build_memory(date, t, &current_body);
                if !memory.text.is_empty() {
                    memories.push(memory);
                }
            }
            current_time = Some(time);
            current_body.clear();
        } else {
            current_body.push(line);
        }
    }

    if let Some(ref t) = current_time {
        let memory = build_memory(date, t, &current_body);
        if !memory.text.is_empty() {
            memories.push(memory);
        }
    }

    memories
}

/// Build a Memory from raw body lines, extracting the tag line if present.
/// The tag line must be the very first line after the timestamp header (no
/// preceding blank line). Body lines after the tag line (or all body lines
/// if no tag line) form the text.
fn build_memory(date: &str, time: &str, body_lines: &[&str]) -> Memory {
    let mut start = 0;

    // Check if the very first line is a tag line (no blank line before it).
    // A tag line starts with '#' immediately followed by a non-space character,
    // distinguishing it from markdown headings like "# Heading".
    let tags = if let Some(first) = body_lines.first() {
        let is_tag_line = first.starts_with('#')
            && first.chars().nth(1).map_or(false, |c| c != ' ');
        if is_tag_line {
            let parsed = parse_tag_line(first);
            if !parsed.is_empty() {
                start = 1; // consume tag line
                parsed
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let text = body_lines[start..]
        .join("\n")
        .trim()
        .to_string();

    let mut m = Memory::new(date, time, &text);
    m.tags = tags;
    m
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
        let m = Memory::new("2026-03-10", "14:32:05", "hello");
        assert_eq!(m.id, "2026-03-10/14:32:05");
        assert_eq!(m.date, "2026-03-10");
        assert_eq!(m.time, "14:32:05");
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

        // id must be "YYYY-MM-DD/HH:MM:SS"
        let parts: Vec<&str> = id.splitn(2, '/').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].len(), 8); // "HH:MM:SS"

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
            "## 09:00:00\n\nearly thought\n\n## 15:30:00\n\nlate thought\n\n",
        ).unwrap();

        let memories = store.list(20).unwrap();
        assert_eq!(memories.len(), 2);
        assert_eq!(memories[0].time, "15:30:00"); // newest first
        assert_eq!(memories[1].time, "09:00:00");
    }

    #[test]
    fn test_list_respects_limit() {
        let (_dir, store) = test_store();
        let date = "2026-03-10";
        let file_path = store.memories_dir.join(format!("{}.md", date));
        std::fs::create_dir_all(&store.memories_dir).unwrap();
        std::fs::write(
            &file_path,
            "## 09:00:00\n\nfirst\n\n## 10:00:00\n\nsecond\n\n## 11:00:00\n\nthird\n\n",
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
            "## 10:00:00\n\nold\n\n",
        ).unwrap();
        std::fs::write(
            store.memories_dir.join("2026-03-10.md"),
            "## 10:00:00\n\nnew\n\n",
        ).unwrap();

        let memories = store.list(20).unwrap();
        assert_eq!(memories[0].date, "2026-03-10"); // newer day first
        assert_eq!(memories[1].date, "2026-03-09");
    }

    #[test]
    fn test_get_returns_correct_memory() {
        let (_dir, store) = test_store();
        let date = "2026-03-10";
        let file_path = store.memories_dir.join(format!("{}.md", date));
        std::fs::create_dir_all(&store.memories_dir).unwrap();
        std::fs::write(&file_path, "## 14:32:05\n\nevent sourcing idea\n\n").unwrap();

        let memory = store.get("2026-03-10/14:32:05").unwrap();
        assert!(memory.is_some());
        assert_eq!(memory.unwrap().text, "event sourcing idea");
    }

    #[test]
    fn test_get_missing_returns_none() {
        let (_dir, store) = test_store();
        let date = "2026-03-10";
        let file_path = store.memories_dir.join(format!("{}.md", date));
        std::fs::create_dir_all(&store.memories_dir).unwrap();
        std::fs::write(&file_path, "## 14:32:05\n\nsome text\n\n").unwrap();

        let memory = store.get("2026-03-10/99:99:99").unwrap();
        assert!(memory.is_none());
    }

    #[test]
    fn test_update_changes_text() {
        let (_dir, store) = test_store();
        let date = "2026-03-10";
        let file_path = store.memories_dir.join(format!("{}.md", date));
        std::fs::create_dir_all(&store.memories_dir).unwrap();
        std::fs::write(&file_path, "## 14:32:05\n\noriginal\n\n").unwrap();

        store.update("2026-03-10/14:32:05", "updated").unwrap();
        let memory = store.get("2026-03-10/14:32:05").unwrap().unwrap();
        assert_eq!(memory.text, "updated");
    }

    #[test]
    fn test_delete_removes_one_entry() {
        let (_dir, store) = test_store();
        let date = "2026-03-10";
        let file_path = store.memories_dir.join(format!("{}.md", date));
        std::fs::create_dir_all(&store.memories_dir).unwrap();
        std::fs::write(
            &file_path,
            "## 14:32:05\n\nhello\n\n## 15:00:00\n\nworld\n\n",
        ).unwrap();

        store.delete("2026-03-10/14:32:05").unwrap();
        assert!(store.get("2026-03-10/14:32:05").unwrap().is_none());
        assert!(store.get("2026-03-10/15:00:00").unwrap().is_some());
    }

    #[test]
    fn test_delete_last_entry_removes_file() {
        let (_dir, store) = test_store();
        let date = "2026-03-10";
        let file_path = store.memories_dir.join(format!("{}.md", date));
        std::fs::create_dir_all(&store.memories_dir).unwrap();
        std::fs::write(&file_path, "## 14:32:05\n\nhello\n\n").unwrap();

        store.delete("2026-03-10/14:32:05").unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_parse_backward_compat_hhmm_format() {
        // Old files written before seconds were added must still parse correctly.
        let (_dir, store) = test_store();
        let date = "2026-03-10";
        let file_path = store.memories_dir.join(format!("{}.md", date));
        std::fs::create_dir_all(&store.memories_dir).unwrap();
        std::fs::write(&file_path, "## 14:32\n\nlegacy entry\n\n").unwrap();

        let memory = store.get("2026-03-10/14:32").unwrap();
        assert!(memory.is_some());
        assert_eq!(memory.unwrap().text, "legacy entry");
    }

    #[test]
    fn test_memory_new_has_empty_tags_by_default() {
        let m = Memory::new("2026-03-10", "14:32:05", "hello");
        assert!(m.tags.is_empty());
    }

    #[test]
    fn test_parse_tag_line_empty() {
        assert!(parse_tag_line("").is_empty());
    }

    #[test]
    fn test_parse_tag_line_no_hash() {
        assert!(parse_tag_line("no hash prefix").is_empty());
    }

    #[test]
    fn test_parse_tag_line_single_tag() {
        assert_eq!(parse_tag_line("#rust"), vec!["rust"]);
    }

    #[test]
    fn test_parse_tag_line_multiple_tags() {
        assert_eq!(parse_tag_line("#rust, #til"), vec!["rust", "til"]);
    }

    #[test]
    fn test_parse_tag_line_multiword_tag() {
        assert_eq!(parse_tag_line("#shower thought, #rust"), vec!["shower thought", "rust"]);
    }

    #[test]
    fn test_parse_tag_line_dedup_and_lowercase() {
        assert_eq!(parse_tag_line("#Rust, #RUST, #rust"), vec!["rust"]);
    }

    #[test]
    fn test_parse_tag_line_whitespace_trimmed() {
        assert_eq!(parse_tag_line("  #rust  ,  #til  "), vec!["rust", "til"]);
    }

    #[test]
    fn test_parse_tag_line_bare_hash_ignored() {
        assert!(parse_tag_line("#").is_empty());
    }

    #[test]
    fn test_parse_tag_line_empty_tag_part_skipped() {
        assert_eq!(parse_tag_line("#, #rust"), vec!["rust"]);
    }

    #[test]
    fn test_parse_tag_line_plain_text_is_not_tag_line() {
        assert!(parse_tag_line("just some text").is_empty());
    }

    #[test]
    fn test_parse_tag_line_strips_underscore() {
        assert_eq!(parse_tag_line("#rust_lang"), vec!["rustlang"]);
    }

    #[test]
    fn test_parse_tag_line_strips_slash() {
        assert_eq!(parse_tag_line("#rust/lang"), vec!["rustlang"]);
    }

    fn parse_file_pub(date: &str, content: &str) -> Vec<Memory> {
        parse_file(date, content)
    }

    #[test]
    fn test_parse_file_with_tag_line() {
        let content = "## 14:32:05\n#rust, #til\n\nborrow checker is tricky\n\n";
        let memories = parse_file_pub("2026-03-11", content);
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].tags, vec!["rust", "til"]);
        assert_eq!(memories[0].text, "borrow checker is tricky");
    }

    #[test]
    fn test_parse_file_without_tag_line_has_empty_tags() {
        let content = "## 15:00:00\n\nwhat do fish think about\n\n";
        let memories = parse_file_pub("2026-03-11", content);
        assert_eq!(memories.len(), 1);
        assert!(memories[0].tags.is_empty());
        assert_eq!(memories[0].text, "what do fish think about");
    }

    #[test]
    fn test_parse_file_multiword_tag() {
        let content = "## 16:00:00\n#shower thought\n\nwhy is water wet\n\n";
        let memories = parse_file_pub("2026-03-11", content);
        assert_eq!(memories[0].tags, vec!["shower thought"]);
        assert_eq!(memories[0].text, "why is water wet");
    }

    #[test]
    fn test_parse_file_hash_in_body_not_treated_as_tag_line() {
        // A markdown heading in the body should NOT be parsed as a tag line
        let content = "## 10:00:00\n\n# not a tag\n\nsome text\n\n";
        let memories = parse_file_pub("2026-03-11", content);
        assert_eq!(memories.len(), 1);
        assert!(memories[0].tags.is_empty());
        assert!(memories[0].text.contains("# not a tag"));
    }

    #[test]
    fn test_parse_file_two_entries_both_tagged() {
        let content = "## 09:00:00\n#morning\n\nfirst\n\n## 10:00:00\n#rust\n\nsecond\n\n";
        let memories = parse_file_pub("2026-03-11", content);
        assert_eq!(memories.len(), 2);
        assert_eq!(memories[0].tags, vec!["morning"]);
        assert_eq!(memories[1].tags, vec!["rust"]);
    }
}
