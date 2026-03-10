use crate::store::Store;
use crate::vectordb::VectorDb;
use anyhow::Result;

pub async fn run(ids: &[String]) -> Result<()> {
    let store = Store::from_config();
    let db = VectorDb::open().await;
    for id in ids {
        store.delete(id)?;
        match &db {
            Ok(db) => {
                if let Err(e) = db.delete(id).await {
                    eprintln!("Warning: failed to delete vector for {}: {}", id, e);
                }
            }
            Err(e) => eprintln!("Warning: vector DB unavailable: {}", e),
        }
        println!("Deleted: {}", id);
    }
    Ok(())
}
