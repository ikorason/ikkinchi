use crate::config::Config;
use crate::embed::EmbedClient;
use crate::store::Store;
use crate::vectordb::VectorDb;
use anyhow::Result;

pub async fn run(text: &str, tags: &[String]) -> Result<()> {
    let config = Config::load()?;
    let store = Store::from_config();
    let id = store.append(text, tags)?;
    println!("Captured: {}", id);

    // Embed and store vector — non-fatal if Ollama is unavailable
    match EmbedClient::from_config(&config) {
        Ok(client) => match client.embed_document(text).await {
            Ok(vec) => match VectorDb::open().await {
                Ok(db) => {
                    if let Err(e) = db.insert(&id, &vec).await {
                        eprintln!("Warning: failed to store vector: {}", e);
                    }
                }
                Err(e) => eprintln!("Warning: failed to open vector DB: {}", e),
            },
            Err(e) => eprintln!("Warning: embedding failed ({}). Run `ikkinchi reindex` to add vector later.", e),
        },
        Err(e) => eprintln!("Warning: embedding unavailable ({}). Run `ikkinchi reindex` to add vector later.", e),
    }

    Ok(())
}
