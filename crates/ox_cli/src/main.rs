use clap::{Parser, Subcommand};
use miette::Result;
use ox_common::fs::FilePath;
use std::path::PathBuf;
use tracing;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check the input file for errors
    Check {
        /// Input file path
        path: PathBuf,
    },
    /// Build the output Rust code
    Build {
        /// Input file path
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize miette
    miette::set_panic_hook();

    let cli = Cli::parse();
    tracing::info!("Oxidizer CLI started");

    match cli.command {
        Commands::Check { path } => {
            ox_orchestrator::check(FilePath::from(path))?;
        }
        Commands::Build { path } => {
            let output = ox_orchestrator::build(FilePath::from(path))?;
            println!("{}", output);
        }
    }

    println!("Hello World from Oxidizer CLI!");
    Ok(())
}
