//! Execução de um processo do Node com ambiente fixo e prazo.
use std::ffi::OsString;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const RUN_TIMEOUT: Duration = Duration::from_secs(20);
const POLL_INTERVAL: Duration = Duration::from_millis(5);
const FIXED_TIMEZONE: &str = "UTC";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub timed_out: bool,
}

#[derive(Debug)]
pub struct RunSpec<'a> {
    pub binary: &'a Path,
    pub args: &'a [OsString],
    pub cwd: &'a Path,
    pub timeout: Duration,
}

fn drain<R: Read + Send + 'static>(mut pipe: R) -> JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut buffer = Vec::new();
        // Leitura interrompida só encurta a captura; o código de saída continua valendo.
        let _ = pipe.read_to_end(&mut buffer);
        buffer
    })
}

/// Roda o binário com ambiente vazio (só `TZ=UTC`), stdin nulo e prazo; estourar o prazo mata o processo.
pub fn run_node(spec: &RunSpec<'_>) -> io::Result<RunOutcome> {
    let mut child = Command::new(spec.binary)
        .args(spec.args)
        .current_dir(spec.cwd)
        .env_clear()
        .env("TZ", FIXED_TIMEZONE)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().map(drain);
    let stderr = child.stderr.take().map(drain);
    let deadline = Instant::now() + spec.timeout;
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            timed_out = true;
            // O processo já estourou o prazo; falha ao matar não muda o veredito.
            let _ = child.kill();
            break child.wait()?;
        }
        thread::sleep(POLL_INTERVAL);
    };
    Ok(RunOutcome {
        exit_code: if timed_out { None } else { status.code() },
        stdout: stdout.and_then(|h| h.join().ok()).unwrap_or_default(),
        stderr: stderr.and_then(|h| h.join().ok()).unwrap_or_default(),
        timed_out,
    })
}
