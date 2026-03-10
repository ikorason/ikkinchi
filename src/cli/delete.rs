use crate::store::Store;
use anyhow::Result;

pub async fn run(ids: &[String]) -> Result<()> {
    let store = Store::from_config();
    for id in ids {
        store.delete(id)?;
        println!("Deleted: {}", id);
    }
    Ok(())
}
