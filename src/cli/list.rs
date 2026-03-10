use crate::config::Config;
use crate::store::Store;
use anyhow::Result;

pub async fn run(count: Option<usize>) -> Result<()> {
    let config = Config::load()?;
    let limit = count.unwrap_or(config.display.list_count);
    let store = Store::from_config();
    let memories = store.list(limit)?;

    if memories.is_empty() {
        println!("No memories yet. Try: ikkinchi add \"your thought\"");
        return Ok(());
    }

    for (i, m) in memories.iter().enumerate() {
        println!("{:>3}  {}  {}", i + 1, m.id, m.text);
    }
    Ok(())
}
