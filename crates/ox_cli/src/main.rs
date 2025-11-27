use clap::{Parser, Subcommand};
use miette::Result;
use ox_common::fs::FilePath;
use std::path::PathBuf;

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
        /// Input file or directory path
        path: PathBuf,
        /// Output directory path (default: ./typerust_output)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize miette
    miette::set_panic_hook();

    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path } => {
            ox_orchestrator::check(FilePath::from(path))?;
        }
        Commands::Build { path, output } => {
            if path.is_dir() {
                let output_dir = output.unwrap_or_else(|| PathBuf::from("./typerust_output"));
                ox_orchestrator::build_project(path, output_dir)?;
                println!("✅ Project built successfully!");
            } else {
                let output_code = ox_orchestrator::build(FilePath::from(path))?;
                println!("{}", output_code);
            }
        }
    }

    Ok(())
}
