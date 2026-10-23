//! Recalcula o sha256 dos arquivos referenciados e compara com o registrado.
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use crate::error::FactsError;
use crate::file_fact::FileFact;
use crate::sha256::{sha256_reader, Sha256Hex};

/// # Errors
/// `E7101` na primeira divergência (arquivo ausente conta como divergência);
/// `E7104` para qualquer outra falha de leitura.
pub fn verify_entries(base: &Path, entries: &[FileFact]) -> Result<(), FactsError> {
    for entry in entries {
        let actual = hash_file(&base.join(entry.path()))?;
        if actual.as_ref() != Some(entry.sha256()) {
            return Err(FactsError::HashMismatch {
                path: entry.path().to_owned(),
                expected: entry.sha256().clone(),
                actual,
            });
        }
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<Option<Sha256Hex>, FactsError> {
    let io_error = |error: std::io::Error| FactsError::Io {
        path: path.to_path_buf(),
        reason: error.to_string(),
    };
    match File::open(path) {
        Ok(mut file) => sha256_reader(&mut file).map(Some).map_err(io_error),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error(error)),
    }
}
