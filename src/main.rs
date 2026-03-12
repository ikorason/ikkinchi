use anyhow::Result;
use clap::Parser;
use ikkinchi::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => ikkinchi::cli::init::run().await?,
        Commands::Add { text, tag } => ikkinchi::cli::add::run(&text, &tag).await?,
        Commands::Search { query } => ikkinchi::cli::search::run(&query).await?,
        Commands::List { count, tag } => ikkinchi::cli::list::run(count, tag).await?,
        Commands::Edit { id, text } => ikkinchi::cli::edit::run(&id, &text).await?,
        Commands::Delete { ids } => ikkinchi::cli::delete::run(&ids).await?,
        Commands::Import { path } => ikkinchi::cli::import::run(&path).await?,
        Commands::Export { format } => {
            ikkinchi::cli::export::run(format.as_deref()).await?
        }
        Commands::Stats => ikkinchi::cli::stats::run().await?,
        Commands::Reindex => ikkinchi::cli::reindex::run().await?,
        Commands::Tui => ikkinchi::tui::run().await?,
    }

    Ok(())
}
