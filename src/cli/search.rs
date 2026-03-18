use crate::config::Config;
use crate::search;
use crate::semantic;
use crate::store::{Memory, Store};
use anyhow::Result;

pub async fn run(query: &str, tag: Option<String>, semantic: bool) -> Result<()> {
    let config = Config::load()?;
    let limit = config.display.list_count;
    // Store is only used by the fuzzy branch.
    // When semantic=true, semantic_search() constructs its own Store internally.
    // Double construction is an accepted tradeoff for symmetric branch structure.
    let store = Store::from_config();

    let mut results: Vec<Memory> = if semantic {
        semantic::semantic_search(query, limit).await?
    } else {
        search::fuzzy_search(&store, query, limit)?
    };

    if results.is_empty() {
        match &tag {
            None => println!("No memories found for: {}", query),
            Some(t) => println!("No results for: {} (tag: #{})", query, t.to_lowercase()),
        }
        return Ok(());
    }

    if let Some(ref t) = tag {
        let t = t.to_lowercase();
        results.retain(|m| m.tags.contains(&t));
        if results.is_empty() {
            println!("No results for: {} (tag: #{})", query, t);
            return Ok(());
        }
    }

    for (i, m) in results.iter().enumerate() {
        println!("{:>3}  {}  {}", i + 1, m.id, m.text);
    }

    Ok(())
}
