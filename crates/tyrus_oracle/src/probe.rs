//! Sonda da tupla de um binário do Node: o oráculo só aceita o motor cuja tupla
//! medida é igual à do `oracle.lock` (D14).
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::error::OracleError;
use crate::lock::OracleLock;
use crate::tuple::{NodeRole, NodeTuple};

pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

const PROBE_SCRIPT: &str = "process.stdout.write(JSON.stringify({node:process.versions.node,\
v8:process.versions.v8,icu:process.versions.icu,unicode:process.versions.unicode,\
tz:process.versions.tz,cldr:process.versions.cldr}))";

fn probe_failed(path: &Path, reason: impl Into<String>) -> OracleError {
    OracleError::NodeProbeFailed {
        path: path.to_path_buf(),
        reason: reason.into(),
    }
}

/// Caminho do binário do papel a partir do ambiente (injetado para não mutar o do processo).
pub fn resolve_binary<F>(role: NodeRole, get_env: F) -> Result<PathBuf, OracleError>
where
    F: Fn(&str) -> Option<OsString>,
{
    match get_env(role.env_var()) {
        Some(value) if !value.is_empty() => Ok(PathBuf::from(value)),
        _ => Err(OracleError::NodeBinaryNotConfigured {
            env_var: role.env_var(),
        }),
    }
}

pub fn parse_probe_output(text: &str) -> Result<NodeTuple, OracleError> {
    let tuple: NodeTuple =
        serde_json::from_str(text.trim()).map_err(|e| OracleError::NodeProbeOutputInvalid {
            reason: e.to_string(),
        })?;
    let blank = [
        &tuple.node,
        &tuple.v8,
        &tuple.icu,
        &tuple.unicode,
        &tuple.tz,
        &tuple.cldr,
    ]
    .into_iter()
    .any(|value| value.trim().is_empty());
    if blank {
        return Err(OracleError::NodeProbeOutputInvalid {
            reason: "campo vazio".to_owned(),
        });
    }
    Ok(tuple)
}

pub fn probe_node(binary: &Path) -> Result<NodeTuple, OracleError> {
    probe_node_with_timeout(binary, DEFAULT_PROBE_TIMEOUT)
}

/// Roda o binário com ambiente vazio (sem `NODE_OPTIONS`, `HOME` ou `PATH`) e exige
/// caminho absoluto: o pin não pode depender de shim nem de variável do chamador.
pub fn probe_node_with_timeout(binary: &Path, timeout: Duration) -> Result<NodeTuple, OracleError> {
    if !binary.is_absolute() {
        return Err(probe_failed(
            binary,
            "o caminho do binário deve ser absoluto",
        ));
    }
    let mut child = Command::new(binary)
        .arg("-e")
        .arg(PROBE_SCRIPT)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| probe_failed(binary, e.to_string()))?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(status)) => return Err(probe_failed(binary, format!("saiu com {status}"))),
            Ok(None) if Instant::now() >= deadline => {
                // O processo já excedeu o prazo; falha ao matar ou reaper não muda o veredito.
                let _ = child.kill();
                let _ = child.wait();
                return Err(probe_failed(
                    binary,
                    format!("tempo esgotado ({timeout:?})"),
                ));
            }
            Ok(None) => std::thread::sleep(POLL_INTERVAL),
            Err(e) => return Err(probe_failed(binary, e.to_string())),
        }
    }
    let mut output = String::new();
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| probe_failed(binary, "stdout indisponível"))?;
    stdout
        .read_to_string(&mut output)
        .map_err(|e| probe_failed(binary, e.to_string()))?;
    parse_probe_output(&output)
}

/// Mede o binário e exige que a tupla seja idêntica à pinada para o papel.
pub fn verify_tuple(
    lock: &OracleLock,
    role: NodeRole,
    binary: &Path,
) -> Result<NodeTuple, OracleError> {
    let measured = probe_node(binary)?;
    let diffs = lock.tuple(role).diff(&measured);
    if diffs.is_empty() {
        Ok(measured)
    } else {
        Err(OracleError::TupleMismatch { role, diffs })
    }
}
