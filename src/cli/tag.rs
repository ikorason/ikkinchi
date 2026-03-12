use crate::store::Store;
use anyhow::Result;

pub async fn run_add(id: &str, tags: &[String]) -> Result<()> {
    let store = Store::from_config();
    store.add_tags(id, tags)?;
    let m = store.get(id)?.expect("memory must exist after add_tags succeeded");
    let tag_display = m.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(", ");
    println!("Tagged {}: {}", id, tag_display);
    Ok(())
}

pub async fn run_remove(id: &str, tags: &[String]) -> Result<()> {
    let store = Store::from_config();
    store.remove_tags(id, tags)?;
    let m = store.get(id)?.expect("memory must exist after remove_tags succeeded");
    if m.tags.is_empty() {
        println!("Tags cleared {}", id);
    } else {
        let tag_display = m.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(", ");
        println!("Tags updated {}: {}", id, tag_display);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    fn tag_display(tags: &[String]) -> String {
        tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(", ")
    }

    #[test]
    fn test_tag_display_single() {
        let tags = vec!["rust".to_string()];
        assert_eq!(tag_display(&tags), "#rust");
    }

    #[test]
    fn test_tag_display_multiple() {
        let tags = vec!["rust".to_string(), "til".to_string()];
        assert_eq!(tag_display(&tags), "#rust, #til");
    }

    #[test]
    fn test_tag_display_empty() {
        let tags: Vec<String> = vec![];
        assert_eq!(tag_display(&tags), "");
    }
}
