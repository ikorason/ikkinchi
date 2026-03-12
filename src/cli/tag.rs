use crate::store::Store;
use anyhow::Result;

pub async fn run_add(id: &str, tags: &[String]) -> Result<()> {
    let store = Store::from_config();
    store.add_tags(id, tags)?;
    let m = store.get(id)?.ok_or_else(|| anyhow::anyhow!("Memory not found: {}", id))?;
    let tag_display = m.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(", ");
    println!("Tagged {}: {}", id, tag_display);
    Ok(())
}

pub async fn run_remove(id: &str, tags: &[String]) -> Result<()> {
    let store = Store::from_config();
    store.remove_tags(id, tags)?;
    let m = store.get(id)?.ok_or_else(|| anyhow::anyhow!("Memory not found: {}", id))?;
    if m.tags.is_empty() {
        println!("Tags cleared {}", id);
    } else {
        let tag_display = m.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(", ");
        println!("Tags updated {}: {}", id, tag_display);
    }
    Ok(())
}
