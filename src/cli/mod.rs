pub mod add;
pub mod delete;
pub mod edit;
pub mod export;
pub mod import;
pub mod init;
pub mod list;
pub mod search;
pub mod stats;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ikkinchi",
    about = "Your second brain — zero-friction thought capture and retrieval",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Set up ~/.ikkinchi/, detect Ollama, create config
    Init,

    /// Capture a thought
    Add {
        /// The thought to capture
        text: String,
    },

    /// Semantic + fuzzy hybrid search
    Search {
        /// Search query
        query: String,
    },

    /// List memories, newest first
    List {
        /// Number of memories to show (default: 20)
        #[arg(short, long)]
        count: Option<usize>,
    },

    /// Replace a memory's content
    Edit {
        /// Memory ID (e.g. 2026-03-10/14:32)
        id: String,
        /// New content
        text: String,
    },

    /// Delete one or more memories
    Delete {
        /// Memory ID(s) to delete
        ids: Vec<String>,
    },

    /// Import .md/.txt files as memories
    Import {
        /// Path to file or directory
        path: PathBuf,
    },

    /// Export memories to stdout
    Export {
        /// Output format (default: markdown)
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
    },

    /// Show brain statistics
    Stats,

    /// Rebuild vectors.db from markdown files
    Reindex,

    /// Launch interactive TUI
    Tui,
}
