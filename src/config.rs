use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub embedding: EmbeddingConfig,
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,
    pub model: String,
    pub url: String,
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub list_count: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            embedding: EmbeddingConfig {
                provider: "ollama".to_string(),
                model: "nomic-embed-text".to_string(),
                url: "http://localhost:11434".to_string(),
                api_key_env: None,
            },
            display: DisplayConfig { list_count: 20 },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let config: Self = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config at {}", path.display()))?;
        Ok(config)
    }
}

pub fn ikkinchi_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot determine home directory")
        .join(".ikkinchi")
}

pub fn config_path() -> PathBuf {
    ikkinchi_dir().join("config.toml")
}

pub fn memories_dir() -> PathBuf {
    ikkinchi_dir().join("memories")
}

pub fn vectors_db_path() -> PathBuf {
    ikkinchi_dir().join("vectors.db")
}
