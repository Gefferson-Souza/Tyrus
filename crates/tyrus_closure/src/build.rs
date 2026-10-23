//! Montagem da árvore de produção: lockfile + `npm ci --omit=dev` (com scripts de
//! instalação) + emit do app em `app/`, e `closure.lock` ao lado da árvore.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tyrus_facts::{canonical_bytes, ClosureLock};

use crate::dev_check::ensure_no_dev;
use crate::error::ClosureError;
use crate::walk::{collect_inventory, lock_from_tree};

pub const APP_DIR: &str = "app";
pub const LOCK_FILE: &str = "closure.lock";
const TREE_DIR: &str = "tree";
const NPM_INPUTS: [(&str, bool); 3] = [
    ("package.json", true),
    ("package-lock.json", true),
    (".npmrc", false),
];

#[derive(Debug)]
pub struct BuildRequest<'a> {
    /// Raiz do projeto com `package.json` e `package-lock.json`.
    pub project: &'a Path,
    /// Diretório `emit/` produzido pelo sidecar ST0.
    pub emit: &'a Path,
    /// Saída: `<out>/tree/` e `<out>/closure.lock`; precisa estar vazia ou não existir.
    pub out: &'a Path,
    pub npm_cache: Option<&'a Path>,
    /// Programa do npm; `npm` do PATH quando ausente.
    pub npm_program: Option<&'a Path>,
}

#[derive(Debug)]
pub struct BuildReport {
    pub tree: PathBuf,
    pub lock_path: PathBuf,
    pub lock: ClosureLock,
    pub skipped_symlinks: Vec<String>,
}

/// # Errors
/// `E7204` se o `npm ci` falhar; `E7201` com dependência de desenvolvimento; `E7205` de E/S.
pub fn build_closure(request: &BuildRequest<'_>) -> Result<BuildReport, ClosureError> {
    let tree = request.out.join(TREE_DIR);
    ensure_empty(request.out)?;
    fs::create_dir_all(&tree).map_err(|error| ClosureError::io(&tree, &error))?;
    copy_npm_inputs(request.project, &tree)?;
    run_npm_ci(request, &tree)?;
    copy_regular_files(request.emit, &tree.join(APP_DIR))?;
    let lockfile = fs::read(tree.join("package-lock.json"))
        .map_err(|error| ClosureError::io(&tree.join("package-lock.json"), &error))?;
    let inventory = collect_inventory(&tree)?;
    ensure_no_dev(&inventory.files, Some(&lockfile))?;
    let (lock, skipped_symlinks) = lock_from_tree(&tree)?;
    let lock_path = request.out.join(LOCK_FILE);
    let bytes = canonical_bytes(&lock)?;
    fs::write(&lock_path, bytes).map_err(|error| ClosureError::io(&lock_path, &error))?;
    Ok(BuildReport {
        tree,
        lock_path,
        lock,
        skipped_symlinks,
    })
}

fn ensure_empty(out: &Path) -> Result<(), ClosureError> {
    let mut entries = match fs::read_dir(out) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(ClosureError::io(out, &error)),
    };
    if entries.next().is_some() {
        return Err(ClosureError::io_reason(
            out,
            "diretório de saída não está vazio",
        ));
    }
    Ok(())
}

fn copy_npm_inputs(project: &Path, tree: &Path) -> Result<(), ClosureError> {
    for (name, required) in NPM_INPUTS {
        let source = project.join(name);
        if source.is_file() {
            fs::copy(&source, tree.join(name))
                .map_err(|error| ClosureError::io(&source, &error))?;
        } else if required {
            return Err(ClosureError::io_reason(
                &source,
                "arquivo obrigatório ausente",
            ));
        }
    }
    Ok(())
}

fn run_npm_ci(request: &BuildRequest<'_>, tree: &Path) -> Result<(), ClosureError> {
    let program = request.npm_program.unwrap_or_else(|| Path::new("npm"));
    let mut command = Command::new(program);
    command
        .args(["ci", "--omit=dev", "--no-audit", "--no-fund"])
        .current_dir(tree);
    if let Some(cache) = request.npm_cache {
        command.arg("--cache").arg(cache);
    }
    let output = command
        .output()
        .map_err(|error| ClosureError::io(program, &error))?;
    if output.status.success() {
        return Ok(());
    }
    Err(ClosureError::Npm {
        status: output.status.to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    })
}

fn copy_regular_files(source: &Path, destination: &Path) -> Result<(), ClosureError> {
    let inventory = collect_inventory(source)?;
    if let Some(link) = inventory.symlinks.first() {
        return Err(ClosureError::io_reason(
            &source.join(link),
            "symlink no emit do app não é suportado",
        ));
    }
    for file in &inventory.files {
        let target = destination.join(file.path());
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| ClosureError::io(parent, &error))?;
        }
        let origin = source.join(file.path());
        fs::copy(&origin, &target).map_err(|error| ClosureError::io(&origin, &error))?;
    }
    Ok(())
}
