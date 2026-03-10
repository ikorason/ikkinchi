use anyhow::{Context, Result};
use rig::client::EmbeddingsClient;
use rig::embeddings::EmbeddingModel as _;
use rig::providers::ollama;

pub struct EmbedClient {
    model: ollama::EmbeddingModel,
    nomic_prefix: bool,
}

impl EmbedClient {
    pub fn from_config(config: &crate::config::Config) -> Result<Self> {
        let url = &config.embedding.url;
        let model_name = &config.embedding.model;
        let ndims = config.embedding.ndims.unwrap_or(768);

        let client = ollama::Client::builder()
            .api_key(rig::client::Nothing)
            .base_url(url)
            .build()
            .context("Failed to build Ollama client")?;

        let model = client.embedding_model_with_ndims(model_name.as_str(), ndims);

        let nomic_prefix = model_name.contains("nomic");

        Ok(Self {
            model,
            nomic_prefix,
        })
    }

    pub async fn embed_document(&self, text: &str) -> Result<Vec<f64>> {
        let input = if self.nomic_prefix {
            format!("search_document: {}", text)
        } else {
            text.to_owned()
        };

        let embeddings = self
            .model
            .embed_texts(vec![input])
            .await
            .with_context(|| "Ollama is not running. Start it with `ollama serve`")?;

        Ok(embeddings.into_iter().next().map(|e| e.vec).unwrap_or_default())
    }

    pub async fn embed_query(&self, text: &str) -> Result<Vec<f64>> {
        let input = if self.nomic_prefix {
            format!("search_query: {}", text)
        } else {
            text.to_owned()
        };

        let embeddings = self
            .model
            .embed_texts(vec![input])
            .await
            .with_context(|| "Ollama is not running. Start it with `ollama serve`")?;

        Ok(embeddings.into_iter().next().map(|e| e.vec).unwrap_or_default())
    }

    pub async fn embed_documents(&self, texts: &[&str]) -> Result<Vec<Vec<f64>>> {
        let inputs: Vec<String> = texts
            .iter()
            .map(|t| {
                if self.nomic_prefix {
                    format!("search_document: {}", t)
                } else {
                    t.to_string()
                }
            })
            .collect();

        let embeddings = self
            .model
            .embed_texts(inputs)
            .await
            .with_context(|| "Ollama is not running. Start it with `ollama serve`")?;

        Ok(embeddings.into_iter().map(|e| e.vec).collect())
    }
}
