use crate::import::chunk_text;
use crate::store::Store;
use anyhow::Result;
use std::path::Path;

fn should_import(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("md") | Some("txt")
    )
}

async fn import_one(store: &Store, path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    for chunk in chunk_text(&content) {
        let id = store.append(&chunk)?;
        println!("Imported: {}", id);
    }
    Ok(())
}

pub async fn run(path: &Path) -> Result<()> {
    let store = Store::from_config();
    if path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && should_import(p))
            .collect();
        entries.sort();
        for entry in entries {
            import_one(&store, &entry).await?;
        }
    } else {
        import_one(&store, path).await?;
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

    #[tokio::test]
    async fn test_import_txt_file() {
        let (_dir, store) = test_store();
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("notes.txt");
        fs::write(&file, "first thought\n\nsecond thought\n\nthird thought").unwrap();

        import_one(&store, &file).await.unwrap();

        let memories = store.list(usize::MAX).unwrap();
        assert_eq!(memories.len(), 3);
    }

    #[tokio::test]
    async fn test_import_md_file() {
        let (_dir, store) = test_store();
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("notes.md");
        fs::write(&file, "# Heading\n\nsome content\n\nanother section").unwrap();

        import_one(&store, &file).await.unwrap();

        let memories = store.list(usize::MAX).unwrap();
        assert_eq!(memories.len(), 3);
    }

    #[tokio::test]
    async fn test_import_empty_file() {
        let (_dir, store) = test_store();
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("empty.txt");
        fs::write(&file, "").unwrap();

        import_one(&store, &file).await.unwrap();

        let memories = store.list(usize::MAX).unwrap();
        assert!(memories.is_empty());
    }
}
