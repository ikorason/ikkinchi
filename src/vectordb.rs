use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

pub struct VectorDb {
    pool: sqlx::SqlitePool,
}

impl VectorDb {
    pub async fn open() -> Result<Self> {
        let path = crate::config::vectors_db_path();
        Self::open_at(&path).await
    }

    pub async fn open_at(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let url = format!("sqlite:{}", path.display());
        let opts = SqliteConnectOptions::from_str(&url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS embeddings (
                id   TEXT PRIMARY KEY,
                vec  BLOB NOT NULL
            )",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn insert(&self, id: &str, vec: &[f64]) -> Result<()> {
        let blob = floats_to_bytes(vec);
        sqlx::query("INSERT OR REPLACE INTO embeddings (id, vec) VALUES (?, ?)")
            .bind(id)
            .bind(blob)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM embeddings WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn load_all(&self) -> Result<Vec<(String, Vec<f64>)>> {
        let rows: Vec<(String, Vec<u8>)> =
            sqlx::query_as("SELECT id, vec FROM embeddings")
                .fetch_all(&self.pool)
                .await?;

        let result = rows
            .into_iter()
            .map(|(id, blob)| {
                let vec = bytes_to_floats(&blob);
                (id, vec)
            })
            .collect();

        Ok(result)
    }

    pub async fn rebuild(&self, entries: &[(String, Vec<f64>)]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM embeddings")
            .execute(&mut *tx)
            .await?;

        for (id, vec) in entries {
            let blob = floats_to_bytes(vec);
            sqlx::query("INSERT INTO embeddings (id, vec) VALUES (?, ?)")
                .bind(id)
                .bind(blob)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn count(&self) -> Result<usize> {
        let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM embeddings")
            .fetch_one(&self.pool)
            .await?;
        Ok(n as usize)
    }
}

fn floats_to_bytes(floats: &[f64]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(floats.len() * 8);
    for f in floats {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

fn bytes_to_floats(bytes: &[u8]) -> Vec<f64> {
    bytes
        .chunks_exact(8)
        .map(|chunk| {
            let arr: [u8; 8] = chunk.try_into().expect("chunk is exactly 8 bytes");
            f64::from_le_bytes(arr)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn make_db(dir: &TempDir) -> VectorDb {
        let path = dir.path().join("test_vectors.db");
        VectorDb::open_at(&path).await.expect("open_at failed")
    }

    fn sample_vec(seed: f64) -> Vec<f64> {
        (0..768).map(|i| seed + i as f64 * 0.001).collect()
    }

    #[tokio::test]
    async fn test_insert_and_load_all() {
        let dir = TempDir::new().unwrap();
        let db = make_db(&dir).await;

        let id = "2026-03-10/14:32";
        let vec = sample_vec(1.0);
        db.insert(id, &vec).await.unwrap();

        let rows = db.load_all().await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, id);

        // Verify round-trip accuracy
        for (orig, loaded) in vec.iter().zip(rows[0].1.iter()) {
            assert_eq!(orig, loaded, "float round-trip mismatch");
        }
    }

    #[tokio::test]
    async fn test_delete() {
        let dir = TempDir::new().unwrap();
        let db = make_db(&dir).await;

        db.insert("id-a", &sample_vec(1.0)).await.unwrap();
        db.insert("id-b", &sample_vec(2.0)).await.unwrap();

        db.delete("id-a").await.unwrap();

        let rows = db.load_all().await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "id-b");
    }

    #[tokio::test]
    async fn test_rebuild() {
        let dir = TempDir::new().unwrap();
        let db = make_db(&dir).await;

        db.insert("old-id", &sample_vec(0.0)).await.unwrap();

        let new_entries = vec![
            ("new-a".to_string(), sample_vec(10.0)),
            ("new-b".to_string(), sample_vec(20.0)),
        ];
        db.rebuild(&new_entries).await.unwrap();

        let rows = db.load_all().await.unwrap();
        assert_eq!(rows.len(), 2);

        let ids: Vec<&str> = rows.iter().map(|(id, _)| id.as_str()).collect();
        assert!(ids.contains(&"new-a"));
        assert!(ids.contains(&"new-b"));
        assert!(!ids.contains(&"old-id"));
    }

    #[tokio::test]
    async fn test_count() {
        let dir = TempDir::new().unwrap();
        let db = make_db(&dir).await;

        db.insert("entry-1", &sample_vec(1.0)).await.unwrap();
        db.insert("entry-2", &sample_vec(2.0)).await.unwrap();

        let n = db.count().await.unwrap();
        assert_eq!(n, 2);
    }

    #[tokio::test]
    async fn test_load_all_empty() {
        let dir = TempDir::new().unwrap();
        let db = make_db(&dir).await;

        let rows = db.load_all().await.unwrap();
        assert!(rows.is_empty());
    }
}
