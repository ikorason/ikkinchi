use crate::store::Store;
use anyhow::Result;

pub async fn run(text: &str) -> Result<()> {
    let store = Store::from_config();
    let id = store.append(text)?;
    println!("Captured: {}", id);
    Ok(())
}
