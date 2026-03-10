use crate::config::Config;
use crate::search::fuzzy_search;
use crate::store::Store;
use anyhow::Result;

pub async fn run(query: &str) -> Result<()> {
    let config = Config::load()?;
    let limit = config.display.list_count;
    let store = Store::from_config();
    let memories = fuzzy_search(&store, query, limit)?;
    if memories.is_empty() {
        println!("No memories found for: {}", query);
        return Ok(());
    }
    for (i, m) in memories.iter().enumerate() {
        println!("{:>3}  {}  {}", i + 1, m.id, m.text);
    }
    Ok(())
}
