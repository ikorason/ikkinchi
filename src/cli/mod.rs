pub mod add;
pub mod delete;
pub mod edit;
pub mod export;
pub mod import;
pub mod init;
pub mod list;
pub mod reindex;
pub mod search;
pub mod stats;
pub mod tag;
pub mod tags;

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
        /// Tag(s) to attach (repeatable: --tag rust --tag til)
        #[arg(short, long)]
        tag: Vec<String>,
    },

    /// Search memories (fuzzy by default, --semantic for semantic search)
    Search {
        /// Search query
        query: String,
        /// Filter results by tag
        #[arg(short, long)]
        tag: Option<String>,
        /// Use semantic search (requires Ollama running)
        #[arg(long)]
        semantic: bool,
    },

    /// List memories, newest first
    List {
        /// Number of memories to show (default: 20)
        #[arg(short, long)]
        count: Option<usize>,
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// Replace a memory's content
    Edit {
        /// Memory ID (e.g. 2026-03-10/14:32:05)
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

    /// Add or remove tags on a memory
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },

    /// List all tags with counts
    Tags,

    /// Show brain statistics
    Stats,

    /// Rebuild vectors.db from markdown files
    Reindex,

    /// Launch interactive TUI
    Tui,
}

#[derive(Subcommand)]
pub enum TagAction {
    /// Add tags to a memory
    Add {
        /// Memory ID (e.g. 2026-03-11/14:32:05)
        id: String,
        /// Tag(s) to add
        tags: Vec<String>,
    },
    /// Remove tags from a memory
    Remove {
        /// Memory ID (e.g. 2026-03-11/14:32:05)
        id: String,
        /// Tag(s) to remove
        tags: Vec<String>,
    },
}
