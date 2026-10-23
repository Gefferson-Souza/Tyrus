//! Execução de um comando com ambiente fixado e tempo limite.
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crate::driver::outcome::Outcome;
use crate::error::OracleError;

const POLL_INTERVAL: Duration = Duration::from_millis(5);
const DRAIN_GRACE: Duration = Duration::from_secs(2);
const FIXED_PATH: &str = "/usr/bin:/bin";

/// Comando do alvo: programa e argumentos fixos; o caminho da sonda entra por último.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
}

impl CommandSpec {
    pub fn parse(text: &str) -> Result<Self, OracleError> {
        let mut words = text.split_whitespace();
        let Some(program) = words.next() else {
            return Err(OracleError::CommandInvalid {
                reason: "comando vazio".to_owned(),
            });
        };
        Ok(Self {
            program: PathBuf::from(program),
            args: words.map(str::to_owned).collect(),
        })
    }
}

/// Ambiente de uma execução: limpo, só o que o oráculo fixa.
#[derive(Debug, Clone, Default)]
pub struct RunEnv {
    pub extra: BTreeMap<String, String>,
}

impl RunEnv {
    fn apply(&self, command: &mut Command) {
        command.env_clear();
        command.env("TZ", "UTC");
        command.env("PATH", FIXED_PATH);
        command.env("NO_COLOR", "1");
        command.envs(&self.extra);
    }
}

fn drain<R: Read + Send + 'static>(mut stream: R) -> Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        // Leitura interrompida mantém os bytes já lidos: o veredito compara o que chegou.
        let _ = stream.read_to_end(&mut bytes);
        let _ = sender.send(bytes);
    });
    receiver
}

fn collect(receiver: &Receiver<Vec<u8>>) -> Vec<u8> {
    receiver.recv_timeout(DRAIN_GRACE).unwrap_or_default()
}

/// Roda `spec` com a sonda como último argumento; estouro de tempo mata o processo.
pub fn run_command(
    spec: &CommandSpec,
    extra_args: &[String],
    cwd: &Path,
    env: &RunEnv,
    timeout: Duration,
) -> Result<Outcome, OracleError> {
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .args(extra_args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    env.apply(&mut command);
    let spawn_failed = |reason: String| OracleError::SpawnFailed {
        program: spec.program.display().to_string(),
        reason,
    };
    let mut child = command.spawn().map_err(|e| spawn_failed(e.to_string()))?;
    let stdout = child.stdout.take().map(drain);
    let stderr = child.stderr.take().map(drain);
    let deadline = Instant::now() + timeout;
    let (status, timed_out) = loop {
        match child.try_wait().map_err(|e| spawn_failed(e.to_string()))? {
            Some(status) => break (status, false),
            None if Instant::now() >= deadline => {
                // O processo já estourou o prazo; falha ao matar não muda o veredito.
                let _ = child.kill();
                let status = child.wait().map_err(|e| spawn_failed(e.to_string()))?;
                break (status, true);
            }
            None => thread::sleep(POLL_INTERVAL),
        }
    };
    Ok(Outcome {
        exit_code: if timed_out { None } else { status.code() },
        stdout: stdout.as_ref().map(collect).unwrap_or_default(),
        stderr: stderr.as_ref().map(collect).unwrap_or_default(),
        timed_out,
    })
}
