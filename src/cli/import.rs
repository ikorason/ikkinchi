use crate::embed::EmbedClient;
use crate::import::chunk_text;
use crate::store::Store;
use crate::vectordb::VectorDb;
use anyhow::Result;
use std::path::Path;

fn should_import(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("md") | Some("txt")
    )
}

async fn import_one(
    store: &Store,
    path: &Path,
    embed: Option<(&EmbedClient, &VectorDb)>,
) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    for chunk in chunk_text(&content) {
        let id = store.append(&chunk)?;
        println!("Imported: {}", id);

        if let Some((client, db)) = embed {
            match client.embed_document(&chunk).await {
                Ok(vec) => {
                    if let Err(e) = db.insert(&id, &vec).await {
                        eprintln!("Warning: failed to store vector for {}: {}", id, e);
                    }
                }
                Err(e) => eprintln!(
                    "Warning: embedding failed for {} ({}). Run `ikkinchi reindex` to add vector later.",
                    id, e
                ),
            }
        }
    }
    Ok(())
}

pub async fn run(path: &Path) -> Result<()> {
    use crate::config::Config;

    let store = Store::from_config();

    // Try to open embed client and vector DB once — non-fatal if unavailable
    let config = Config::load().ok();
    let embed_client = config
        .as_ref()
        .and_then(|c| EmbedClient::from_config(c).ok());
    let vector_db = match VectorDb::open().await {
        Ok(db) => Some(db),
        Err(e) => {
            eprintln!("Warning: failed to open vector DB: {}", e);
            None
        }
    };
    let embed = match (embed_client.as_ref(), vector_db.as_ref()) {
        (Some(c), Some(db)) => Some((c, db)),
        _ => None,
    };

    if path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && should_import(p))
            .collect();
        entries.sort();
        for entry in entries {
            import_one(&store, &entry, embed).await?;
        }
    } else {
        import_one(&store, path, embed).await?;
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

        import_one(&store, &file, None).await.unwrap();

        let memories = store.list(usize::MAX).unwrap();
        assert_eq!(memories.len(), 3);
    }

    #[tokio::test]
    async fn test_import_md_file() {
        let (_dir, store) = test_store();
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("notes.md");
        fs::write(&file, "# Heading\n\nsome content\n\nanother section").unwrap();

        import_one(&store, &file, None).await.unwrap();

        let memories = store.list(usize::MAX).unwrap();
        assert_eq!(memories.len(), 3);
    }

    #[tokio::test]
    async fn test_import_empty_file() {
        let (_dir, store) = test_store();
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("empty.txt");
        fs::write(&file, "").unwrap();

        import_one(&store, &file, None).await.unwrap();

        let memories = store.list(usize::MAX).unwrap();
        assert!(memories.is_empty());
    }
}
