use crate::config::Config;
use crate::embed::EmbedClient;
use crate::store::Store;
use crate::vectordb::VectorDb;
use anyhow::Result;

pub async fn run() -> Result<()> {
    let config = Config::load()?;
    let store = Store::from_config();
    let embed_client = EmbedClient::from_config(&config)?;
    let db = VectorDb::open().await?;

    let memories = store.list(usize::MAX)?;
    if memories.is_empty() {
        println!("No memories to index.");
        return Ok(());
    }

    let texts: Vec<&str> = memories.iter().map(|m| m.text.as_str()).collect();
    let vecs = embed_client.embed_documents(&texts).await?;

    let entries: Vec<(String, Vec<f64>)> = memories
        .iter()
        .zip(vecs.into_iter())
        .map(|(m, vec)| (m.id.clone(), vec))
        .collect();

    db.rebuild(&entries).await?;

    println!("Reindexed {} memories.", entries.len());
    Ok(())
}
