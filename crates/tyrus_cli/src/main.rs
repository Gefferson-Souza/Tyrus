use clap::{Parser, Subcommand};
use miette::Result;
use std::path::PathBuf;
use tyrus_common::fs::FilePath;

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
        /// Output directory path (default: ./tyrus_output)
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
            tyrus_orchestrator::check(FilePath::from(path))?;
        }
        Commands::Build { path, output } => {
            if path.is_dir() {
                let output_dir = output.unwrap_or_else(|| PathBuf::from("./tyrus_output"));
                tyrus_orchestrator::build_project(path, output_dir)?;
                println!("✅ Project built successfully!");
            } else {
                let output_code = tyrus_orchestrator::build(FilePath::from(path))?;
                if let Some(output_path) = output {
                    if let Some(parent) = output_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| miette::miette!("Failed to create directory: {}", e))?;
                    }
                    std::fs::write(&output_path, output_code)
                        .map_err(|e| miette::miette!("Failed to write output file: {}", e))?;
                    eprintln!("✅ Built to {:?}", output_path);
                } else {
                    println!("{}", output_code);
                }
            }
        }
    }

    Ok(())
}
