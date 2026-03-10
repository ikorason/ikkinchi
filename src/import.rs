pub fn chunk_text(content: &str) -> Vec<String> {
    content
        .split("\n\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub async fn import_file(_path: &std::path::Path) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_empty() {
        assert!(chunk_text("").is_empty());
    }

    #[test]
    fn test_chunk_text_single_paragraph() {
        assert_eq!(chunk_text("hello world"), vec!["hello world"]);
    }

    #[test]
    fn test_chunk_text_multiple_paragraphs() {
        let result = chunk_text("first\n\nsecond\n\nthird");
        assert_eq!(result, vec!["first", "second", "third"]);
    }

    #[test]
    fn test_chunk_text_skips_blank_chunks() {
        let result = chunk_text("first\n\n\n\nsecond");
        assert_eq!(result, vec!["first", "second"]);
    }

    #[test]
    fn test_chunk_text_trims_whitespace() {
        let result = chunk_text("  hello  \n\n  world  ");
        assert_eq!(result, vec!["hello", "world"]);
    }
}
