#![forbid(unsafe_code)]

use std::io;
use std::process::ExitCode;

use tyrus_oracle::cli::{run, EXIT_ORACLE_ERROR};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut stdout, mut stderr) = (io::stdout().lock(), io::stderr().lock());
    match run(
        &args,
        &|name| std::env::var_os(name),
        &mut stdout,
        &mut stderr,
    ) {
        Ok(code) => ExitCode::from(code),
        Err(_) => ExitCode::from(EXIT_ORACLE_ERROR),
    }
}
