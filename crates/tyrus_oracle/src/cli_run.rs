//! Subcomando `tyrus-oracle run`: o driver Node × hospedado × promovido.
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use crate::driver::corpus::default_probes_dir;
use crate::driver::profile::Profile;
use crate::driver::record::ShimCanary;
use crate::driver::run::{run_corpus, RunConfig, RunSummary};
use crate::driver::runner::CommandSpec;
use crate::error::OracleError;
use crate::lock::OracleLock;
use crate::probe::{resolve_binary, verify_tuple};
use crate::tuple::NodeRole;

pub const EXIT_OK: u8 = 0;
pub const EXIT_NOT_GREEN: u8 = 1;
pub const EXIT_ORACLE_ERROR: u8 = 2;
pub const EXIT_USAGE: u8 = 64;

pub const RUN_USAGE: &str = "uso: tyrus-oracle run --corpus <seed|live> --reference node \
--hosted-cmd <cmd> [--promoted-cmd <cmd>] [--profile exact|legacy_trim] \
[--tuple-role reference|seed] [--out <dir>] [--lock <caminho>] [--probes-dir <dir>] \
[--shim <arquivo>] [--filter <prefixo>] [--hosted-timeout-ms <n>] [--self-check]";

const DEFAULT_LOCK_PATH: &str = "oracle/oracle.lock";
const DEFAULT_SHIM_PATH: &str = "oracle/shim/fix-env.js";
const DEFAULT_OUT_ROOT: &str = "oracle/out";
const DEFAULT_HOSTED_TIMEOUT_MS: u64 = 10_000;
const REFERENCE_TIMEOUT: Duration = Duration::from_secs(20);

struct RunOptions {
    corpus: Option<String>,
    reference: Option<String>,
    hosted: Option<String>,
    promoted: Option<String>,
    profile: Profile,
    role: NodeRole,
    out: Option<PathBuf>,
    lock: PathBuf,
    probes_dir: Option<PathBuf>,
    shim: PathBuf,
    filter: Option<String>,
    hosted_timeout: Duration,
    self_check: bool,
}

fn apply_flag(options: &mut RunOptions, flag: &str, value: &str) -> Result<(), String> {
    match flag {
        "--corpus" => options.corpus = Some(value.to_owned()),
        "--reference" => options.reference = Some(value.to_owned()),
        "--hosted-cmd" => options.hosted = Some(value.to_owned()),
        "--promoted-cmd" => options.promoted = Some(value.to_owned()),
        "--profile" => options.profile = value.parse()?,
        "--tuple-role" => options.role = value.parse()?,
        "--out" => options.out = Some(PathBuf::from(value)),
        "--lock" => options.lock = PathBuf::from(value),
        "--probes-dir" => options.probes_dir = Some(PathBuf::from(value)),
        "--shim" => options.shim = PathBuf::from(value),
        "--filter" => options.filter = Some(value.to_owned()),
        "--hosted-timeout-ms" => {
            let millis: u64 = value
                .parse()
                .map_err(|_| format!("`{value}` não é um número de milissegundos"))?;
            options.hosted_timeout = Duration::from_millis(millis);
        }
        other => return Err(format!("opção desconhecida `{other}`")),
    }
    Ok(())
}

fn parse_run(args: &[String]) -> Result<RunOptions, String> {
    let mut options = RunOptions {
        corpus: None,
        reference: None,
        hosted: None,
        promoted: None,
        profile: Profile::Exact,
        role: NodeRole::Reference,
        out: None,
        lock: PathBuf::from(DEFAULT_LOCK_PATH),
        probes_dir: None,
        shim: PathBuf::from(DEFAULT_SHIM_PATH),
        filter: None,
        hosted_timeout: Duration::from_millis(DEFAULT_HOSTED_TIMEOUT_MS),
        self_check: false,
    };
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        if flag == "--self-check" {
            options.self_check = true;
            continue;
        }
        let value = rest
            .next()
            .ok_or_else(|| format!("{flag} exige um valor"))?;
        apply_flag(&mut options, flag, value)?;
    }
    Ok(options)
}

fn required<'a>(value: Option<&'a String>, flag: &str) -> Result<&'a String, String> {
    value.ok_or_else(|| format!("{flag} é obrigatório"))
}

fn absolute(path: PathBuf) -> PathBuf {
    std::path::absolute(&path).unwrap_or(path)
}

fn build_config<F>(
    options: &RunOptions,
    get_env: &F,
) -> Result<Result<RunConfig, OracleError>, String>
where
    F: Fn(&str) -> Option<OsString>,
{
    let corpus = required(options.corpus.as_ref(), "--corpus")?;
    let reference = required(options.reference.as_ref(), "--reference")?;
    let hosted = required(options.hosted.as_ref(), "--hosted-cmd")?;
    if reference != "node" {
        return Err(format!(
            "--reference `{reference}` não suportado (use `node`)"
        ));
    }
    if !matches!(corpus.as_str(), "seed" | "live") {
        return Err(format!(
            "--corpus `{corpus}` desconhecido (use `seed` ou `live`)"
        ));
    }
    Ok(assemble(options, corpus, hosted, get_env))
}

fn assemble<F>(
    options: &RunOptions,
    corpus: &str,
    hosted: &str,
    get_env: &F,
) -> Result<RunConfig, OracleError>
where
    F: Fn(&str) -> Option<OsString>,
{
    let lock = OracleLock::load(&options.lock)?;
    let binary = resolve_binary(options.role, get_env)?;
    let tuple = verify_tuple(&lock, options.role, &binary)?;
    let promoted = options
        .promoted
        .as_deref()
        .map(CommandSpec::parse)
        .transpose()?;
    let out_dir = options.out.clone().unwrap_or_else(|| {
        PathBuf::from(DEFAULT_OUT_ROOT)
            .join(corpus)
            .join(options.profile.as_str())
    });
    Ok(RunConfig {
        corpus: corpus.to_owned(),
        probes_dir: absolute(
            options
                .probes_dir
                .clone()
                .unwrap_or_else(|| default_probes_dir(corpus)),
        ),
        out_dir,
        profile: options.profile,
        tuple_role: options.role,
        tuple,
        reference_node: binary,
        hosted: CommandSpec::parse(hosted)?,
        promoted,
        shim: absolute(options.shim.clone()),
        self_check: options.self_check,
        filter: options.filter.clone(),
        reference_timeout: REFERENCE_TIMEOUT,
        hosted_timeout: options.hosted_timeout,
    })
}

fn report(summary: &RunSummary, out: &mut dyn Write) -> io::Result<u8> {
    let total = summary.records.len();
    writeln!(
        out,
        "run: {} sondas, {} verdes, {} não verdes",
        total,
        summary.green_count(),
        total - summary.green_count()
    )?;
    if summary
        .records
        .iter()
        .any(|record| record.shim_canary == ShimCanary::NotApplied)
    {
        writeln!(
            out,
            "aviso: o alvo não aplicou o shim de determinismo (canário falhou)"
        )?;
    }
    Ok(if summary.all_green() {
        EXIT_OK
    } else {
        EXIT_NOT_GREEN
    })
}

pub fn run_command<F>(
    args: &[String],
    get_env: &F,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> io::Result<u8>
where
    F: Fn(&str) -> Option<OsString>,
{
    let config = parse_run(args).and_then(|options| build_config(&options, get_env));
    let config = match config {
        Ok(config) => config,
        Err(message) => {
            writeln!(err, "{message}\n{RUN_USAGE}")?;
            return Ok(EXIT_USAGE);
        }
    };
    match config.and_then(|cfg| run_corpus(&cfg)) {
        Ok(summary) => report(&summary, out),
        Err(error) => {
            writeln!(err, "{}: {error}", error.stable_code())?;
            Ok(EXIT_ORACLE_ERROR)
        }
    }
}
