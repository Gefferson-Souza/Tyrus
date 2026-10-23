//! Inventário determinístico: só arquivos regulares, caminho relativo com `/`, ordem por
//! bytes e modo normalizado. Symlinks ficam fora do lock mas são sempre listados.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use tyrus_facts::{sha256_hex, ClosureFile, ClosureLock, FileMode};

use crate::error::ClosureError;

const EXEC_BITS: u32 = 0o111;

#[derive(Debug)]
pub struct Inventory {
    pub files: Vec<ClosureFile>,
    pub symlinks: Vec<String>,
}

/// # Errors
/// `E7205` para falha de leitura, nome não UTF-8 ou tipo de arquivo não suportado.
pub fn collect_inventory(root: &Path) -> Result<Inventory, ClosureError> {
    let mut files = Vec::new();
    let mut symlinks = Vec::new();
    let mut pending = vec![(root.to_path_buf(), String::new())];
    while let Some((dir, prefix)) = pending.pop() {
        let entries = fs::read_dir(&dir).map_err(|error| ClosureError::io(&dir, &error))?;
        for entry in entries {
            let entry = entry.map_err(|error| ClosureError::io(&dir, &error))?;
            let path = entry.path();
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| ClosureError::io_reason(&path, "nome de arquivo que não é UTF-8"))?;
            let relative = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            let kind = entry
                .file_type()
                .map_err(|error| ClosureError::io(&path, &error))?;
            if kind.is_dir() {
                pending.push((path, relative));
            } else if kind.is_symlink() {
                symlinks.push(relative);
            } else if kind.is_file() {
                files.push(closure_file(&path, &relative)?);
            } else {
                return Err(ClosureError::io_reason(
                    &path,
                    "tipo de arquivo não suportado",
                ));
            }
        }
    }
    files.sort_by(|left, right| left.path().cmp(right.path()));
    symlinks.sort();
    Ok(Inventory { files, symlinks })
}

fn closure_file(path: &Path, relative: &str) -> Result<ClosureFile, ClosureError> {
    let content = fs::read(path).map_err(|error| ClosureError::io(path, &error))?;
    let metadata = fs::metadata(path).map_err(|error| ClosureError::io(path, &error))?;
    let mode = if metadata.permissions().mode() & EXEC_BITS == 0 {
        FileMode::Regular
    } else {
        FileMode::Executable
    };
    Ok(ClosureFile::new(relative, mode, sha256_hex(&content))?)
}

/// # Errors
/// Erros de `collect_inventory`; `E7103` se o inventário violar a ordenação estrita.
pub fn lock_from_tree(root: &Path) -> Result<(ClosureLock, Vec<String>), ClosureError> {
    let inventory = collect_inventory(root)?;
    Ok((ClosureLock::new(inventory.files)?, inventory.symlinks))
}
