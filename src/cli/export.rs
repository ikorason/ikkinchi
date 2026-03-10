use crate::store::{Memory, Store};
use anyhow::Result;

pub fn format_memories(memories: &[Memory], format: Option<&str>) -> Result<String> {
    match format {
        None | Some("markdown") => {
            let mut out = String::new();
            for m in memories {
                out.push_str(&format!("## {}\n\n{}\n\n", m.id, m.text));
            }
            Ok(out.trim_end().to_string())
        }
        Some("json") => {
            let values: Vec<serde_json::Value> = memories
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "id": m.id,
                        "date": m.date,
                        "time": m.time,
                        "text": m.text,
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&values)?)
        }
        Some(other) => anyhow::bail!("Unknown format: {}. Use 'json'.", other),
    }
}

pub async fn run(format: Option<&str>) -> Result<()> {
    let store = Store::from_config();
    let mut memories = store.list(usize::MAX)?;
    memories.reverse(); // oldest first
    if memories.is_empty() {
        return Ok(());
    }
    let output = format_memories(&memories, format)?;
    println!("{}", output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Memory;

    fn make_memory(date: &str, time: &str, text: &str) -> Memory {
        Memory::new(date, time, text)
    }

    #[test]
    fn test_export_empty_returns_empty_string() {
        let result = format_memories(&[], None).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_export_markdown_single() {
        let memories = vec![make_memory("2026-03-10", "14:32", "event sourcing idea")];
        let result = format_memories(&memories, None).unwrap();
        assert_eq!(result, "## 2026-03-10/14:32\n\nevent sourcing idea");
    }

    #[test]
    fn test_export_markdown_multiple() {
        let memories = vec![
            make_memory("2026-03-10", "14:32", "first"),
            make_memory("2026-03-10", "15:00", "second"),
        ];
        let result = format_memories(&memories, None).unwrap();
        assert_eq!(result, "## 2026-03-10/14:32\n\nfirst\n\n## 2026-03-10/15:00\n\nsecond");
    }

    #[test]
    fn test_export_json_single() {
        let memories = vec![make_memory("2026-03-10", "14:32", "event sourcing idea")];
        let result = format_memories(&memories, Some("json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed[0]["id"], "2026-03-10/14:32");
        assert_eq!(parsed[0]["text"], "event sourcing idea");
    }

    #[test]
    fn test_export_json_multiple() {
        let memories = vec![
            make_memory("2026-03-10", "14:32", "first"),
            make_memory("2026-03-10", "15:00", "second"),
        ];
        let result = format_memories(&memories, Some("json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        assert_eq!(parsed[1]["text"], "second");
    }

    #[test]
    fn test_export_unknown_format_errors() {
        let result = format_memories(&[], Some("csv"));
        assert!(result.is_err());
    }
}
