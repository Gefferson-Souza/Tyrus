#![forbid(unsafe_code)]
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the CLI's job is printing to the terminal"
)]

use clap::{Parser, Subcommand};
use miette::Result;
use std::path::PathBuf;

mod commands;
mod ui;

#[derive(Parser)]
#[command(
    name = "tyrus",
    author,
    version,
    about = "Tyrus — TypeScript to Rust Compiler",
    long_about = "A source-to-source compiler that converts Oxidizable TypeScript into memory-safe Rust.\nSafe Transpilation · Zero Runtime · Semantic Preservation"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Suppress banner and decorations
    #[arg(long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze TypeScript for Oxidizable compatibility
    Check {
        /// Input file path
        path: PathBuf,
        /// Output diagnostics as JSON (stable envelope with schemaVersion)
        #[arg(long)]
        json: bool,
        /// Promote diagnostics (warnings) to errors. Default: only hard
        /// errors (UseOfVar/UseOfAny/UseOfEval/etc.) cause non-zero exit.
        #[arg(long)]
        strict: bool,
    },
    /// Transpile TypeScript to Rust source code
    Build {
        /// Input file or directory path
        path: PathBuf,
        /// Output directory (default: ./`tyrus_output`)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Transpile and compile to native binary
    Compile {
        /// Input file or directory path
        path: PathBuf,
        /// Output directory (default: ./`tyrus_output`)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Build in release mode with optimizations
        #[arg(long)]
        release: bool,
    },
    /// Transpile, compile, and execute
    Run {
        /// Input file or directory path
        path: PathBuf,
        /// Output directory (default: ./`tyrus_output`)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Build the production closure (--build) or validate it against an oracle load trace (--validate)
    Closure(commands::closure::ClosureArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    miette::set_panic_hook();

    let cli = Cli::parse();

    if !cli.quiet {
        ui::banner::print_banner();
    }

    match cli.command {
        Commands::Check { path, json, strict } => commands::check::execute(&path, json, strict),
        Commands::Build { path, output } => commands::build::execute(&path, output),
        Commands::Compile {
            path,
            output,
            release,
        } => commands::compile::execute(&path, output, release),
        Commands::Run { path, output } => commands::run::execute(&path, output),
        Commands::Closure(args) => commands::closure::execute(&args),
    }
}
