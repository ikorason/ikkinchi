use anyhow::Result;
use clap::Parser;
use ikkinchi::cli::{Cli, Commands, TagAction};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => ikkinchi::cli::init::run().await?,
        Commands::Add { text, tag } => ikkinchi::cli::add::run(&text, &tag).await?,
        Commands::Search { query, tag, semantic } => ikkinchi::cli::search::run(&query, tag, semantic).await?,
        Commands::List { count, tag } => ikkinchi::cli::list::run(count, tag).await?,
        Commands::Edit { id, text } => ikkinchi::cli::edit::run(&id, &text).await?,
        Commands::Delete { ids } => ikkinchi::cli::delete::run(&ids).await?,
        Commands::Import { path } => ikkinchi::cli::import::run(&path).await?,
        Commands::Export { format } => {
            ikkinchi::cli::export::run(format.as_deref()).await?
        }
        Commands::Tag { action } => match action {
            TagAction::Add { id, tags } => ikkinchi::cli::tag::run_add(&id, &tags).await?,
            TagAction::Remove { id, tags } => ikkinchi::cli::tag::run_remove(&id, &tags).await?,
        },
        Commands::Tags => ikkinchi::cli::tags::run().await?,
        Commands::Stats => ikkinchi::cli::stats::run().await?,
        Commands::Reindex => ikkinchi::cli::reindex::run().await?,
        Commands::Tui => ikkinchi::tui::run().await?,
    }

    Ok(())
}
