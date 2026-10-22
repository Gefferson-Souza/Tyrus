//! Linha de comando do oráculo: `tyrus-oracle lock verify`.
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::error::OracleError;
use crate::lock::OracleLock;
use crate::probe::{resolve_binary, verify_tuple};
use crate::tuple::NodeRole;

pub const EXIT_OK: u8 = 0;
pub const EXIT_ORACLE_ERROR: u8 = 2;
pub const EXIT_USAGE: u8 = 64;
pub const EXIT_IO: u8 = 74;

const DEFAULT_LOCK_PATH: &str = "oracle/oracle.lock";
const USAGE: &str = "uso: tyrus-oracle lock verify [--lock <caminho>] [--role reference|seed|all]";

struct VerifyOptions {
    lock: PathBuf,
    roles: Vec<NodeRole>,
}

fn parse_verify(args: &[String]) -> Result<VerifyOptions, String> {
    let mut lock = PathBuf::from(DEFAULT_LOCK_PATH);
    let mut roles = vec![NodeRole::Reference];
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        let value = rest
            .next()
            .ok_or_else(|| format!("{flag} exige um valor"))?;
        match flag.as_str() {
            "--lock" => lock = PathBuf::from(value),
            "--role" if value == "all" => roles = vec![NodeRole::Reference, NodeRole::Seed],
            "--role" => roles = vec![value.parse::<NodeRole>()?],
            other => return Err(format!("opção desconhecida `{other}`")),
        }
    }
    Ok(VerifyOptions { lock, roles })
}

fn verify<F>(options: &VerifyOptions, get_env: &F) -> Result<Vec<String>, OracleError>
where
    F: Fn(&str) -> Option<OsString>,
{
    let lock = OracleLock::load(&options.lock)?;
    let mut lines = Vec::new();
    for role in &options.roles {
        let binary = resolve_binary(*role, get_env)?;
        let t = verify_tuple(&lock, *role, &binary)?;
        lines.push(format!(
            "OK {role} node={} v8={} icu={} unicode={} tz={} cldr={}",
            t.node, t.v8, t.icu, t.unicode, t.tz, t.cldr
        ));
    }
    Ok(lines)
}

/// Executa o comando e devolve o código de saída; falhas de E/S nos fluxos sobem como `Err`.
pub fn run<F>(
    args: &[String],
    get_env: &F,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> io::Result<u8>
where
    F: Fn(&str) -> Option<OsString>,
{
    let parsed = match args {
        [group, command, rest @ ..] if group == "lock" && command == "verify" => parse_verify(rest),
        _ => Err("comando desconhecido".to_owned()),
    };
    let options = match parsed {
        Ok(options) => options,
        Err(message) => {
            writeln!(err, "{message}\n{USAGE}")?;
            return Ok(EXIT_USAGE);
        }
    };
    match verify(&options, get_env) {
        Ok(lines) => {
            for line in lines {
                writeln!(out, "{line}")?;
            }
            Ok(EXIT_OK)
        }
        Err(error) => {
            writeln!(err, "{}: {error}", error.stable_code())?;
            Ok(EXIT_ORACLE_ERROR)
        }
    }
}
