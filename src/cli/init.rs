use anyhow::Result;
use std::path::Path;

/// Sets up the ikkinchi directory at `dir`.
/// Returns true if newly initialized, false if config already existed.
pub fn setup(dir: &Path) -> Result<bool> {
    let memories = dir.join("memories");
    std::fs::create_dir_all(&memories)?;

    let config_path = dir.join("config.toml");
    if config_path.exists() {
        return Ok(false);
    }

    let config = crate::config::Config::default();
    let toml = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, toml)?;
    Ok(true)
}

pub async fn run() -> Result<()> {
    let dir = crate::config::ikkinchi_dir();
    if setup(&dir)? {
        println!("Initialized ikkinchi at: ~/.ikkinchi/");
    } else {
        println!("Already initialized. Config at: ~/.ikkinchi/config.toml");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_setup_creates_directories() {
        let tmp = TempDir::new().unwrap();
        setup(tmp.path()).unwrap();
        assert!(tmp.path().join("memories").exists());
    }

    #[test]
    fn test_setup_writes_config() {
        let tmp = TempDir::new().unwrap();
        setup(tmp.path()).unwrap();
        let config_path = tmp.path().join("config.toml");
        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("ollama"));
        assert!(content.contains("nomic-embed-text"));
    }

    #[test]
    fn test_setup_returns_true_on_fresh_init() {
        let tmp = TempDir::new().unwrap();
        let result = setup(tmp.path()).unwrap();
        assert!(result);
    }

    #[test]
    fn test_setup_returns_false_if_already_initialized() {
        let tmp = TempDir::new().unwrap();
        setup(tmp.path()).unwrap();
        let result = setup(tmp.path()).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_setup_idempotent_does_not_overwrite_config() {
        let tmp = TempDir::new().unwrap();
        setup(tmp.path()).unwrap();
        // Modify config
        std::fs::write(tmp.path().join("config.toml"), "custom = true").unwrap();
        // Re-run setup — should not overwrite
        setup(tmp.path()).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("config.toml")).unwrap();
        assert_eq!(content, "custom = true");
    }
}
