use crate::store::Store;
use anyhow::Result;

pub async fn run(id: &str, text: &str) -> Result<()> {
    let store = Store::from_config();
    store.update(id, text)?;
    println!("Updated: {}", id);
    Ok(())
}
