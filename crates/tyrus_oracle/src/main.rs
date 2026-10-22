#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::process::ExitCode;

use tyrus_oracle::cli::{run, EXIT_IO, EXIT_USAGE};

fn main() -> ExitCode {
    let mut args = Vec::new();
    for raw in std::env::args_os().skip(1) {
        let Ok(arg) = raw.into_string() else {
            let _ = writeln!(io::stderr(), "argumento que não é UTF-8 válido");
            return ExitCode::from(EXIT_USAGE);
        };
        args.push(arg);
    }
    let (mut stdout, mut stderr) = (io::stdout().lock(), io::stderr().lock());
    match run(
        &args,
        &|name| std::env::var_os(name),
        &mut stdout,
        &mut stderr,
    ) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let _ = writeln!(io::stderr(), "falha de E/S nos fluxos: {error}");
            ExitCode::from(EXIT_IO)
        }
    }
}
