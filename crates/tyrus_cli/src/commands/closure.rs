use std::fs;
use std::path::{Path, PathBuf};

use clap::{ArgGroup, Args};
use miette::{miette, Result};
use tyrus_closure::{build_closure, parse_trace, validate_trace, BuildRequest, ClosureError};
use tyrus_facts::parse_closure_lock;

#[derive(Args)]
#[command(group(ArgGroup::new("mode").required(true).args(["build", "validate"])))]
pub(crate) struct ClosureArgs {
    /// Assemble the production tree (npm ci --omit=dev + app emit) and write closure.lock
    #[arg(long, requires_all = ["project", "emit", "out"])]
    build: bool,
    /// Check the closure against an oracle module-load trace (jsonl)
    #[arg(long, requires_all = ["root", "lock", "trace"])]
    validate: bool,
    /// Project root with package.json and package-lock.json (--build)
    #[arg(long)]
    project: Option<PathBuf>,
    /// ST0 emit directory of the app (--build)
    #[arg(long)]
    emit: Option<PathBuf>,
    /// Empty output directory: receives tree/ and closure.lock (--build)
    #[arg(long)]
    out: Option<PathBuf>,
    /// npm cache directory (--build)
    #[arg(long)]
    npm_cache: Option<PathBuf>,
    /// Production tree to validate (--validate)
    #[arg(long)]
    root: Option<PathBuf>,
    /// closure.lock to validate against (--validate)
    #[arg(long)]
    lock: Option<PathBuf>,
    /// Module-load trace written by the oracle hook (--validate)
    #[arg(long)]
    trace: Option<PathBuf>,
}

pub(crate) fn execute(args: &ClosureArgs) -> Result<()> {
    let outcome = match (
        &args.project,
        &args.emit,
        &args.out,
        &args.root,
        &args.lock,
        &args.trace,
    ) {
        (Some(project), Some(emit), Some(out), _, _, _) if args.build => {
            build(project, emit, out, args.npm_cache.as_deref())
        }
        (_, _, _, Some(root), Some(lock), Some(trace)) if args.validate => {
            validate(root, lock, trace)
        }
        _ => return Err(miette!("argumentos incompletos para o modo escolhido")),
    };
    outcome.map_err(|error| report(&error))
}

/// Escreve a mensagem inteira em stderr: o miette quebraria caminhos longos no meio.
fn report(error: &ClosureError) -> miette::Report {
    eprintln!("{}: {error}", error.stable_code());
    if let ClosureError::TraceMissing { paths } = error {
        for path in paths {
            eprintln!("  ausente: {path}");
        }
    }
    miette!("{}", error.stable_code())
}

fn build(
    project: &Path,
    emit: &Path,
    out: &Path,
    npm_cache: Option<&Path>,
) -> Result<(), ClosureError> {
    let report = build_closure(&BuildRequest {
        project,
        emit,
        out,
        npm_cache,
        npm_program: None,
    })?;
    println!(
        "closure: {} arquivos em {} ({})",
        report.lock.files().len(),
        report.tree.display(),
        report.lock_path.display()
    );
    for link in &report.skipped_symlinks {
        println!("closure: symlink fora do lock: {link}");
    }
    Ok(())
}

fn validate(root: &Path, lock: &Path, trace: &Path) -> Result<(), ClosureError> {
    let lock_bytes = read(lock)?;
    let trace_text = String::from_utf8_lossy(&read(trace)?).into_owned();
    let report = validate_trace(
        root,
        &parse_closure_lock(&lock_bytes)?,
        &parse_trace(&trace_text)?,
    )?;
    println!(
        "closure: ok ({} arquivos do rastro no fecho, {} externos ignorados)",
        report.checked, report.external
    );
    Ok(())
}

fn read(path: &Path) -> Result<Vec<u8>, ClosureError> {
    fs::read(path).map_err(|error| ClosureError::Io {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}
