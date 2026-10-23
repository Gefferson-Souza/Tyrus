//! `tyrus-archive inspect [--json] <arquivo>`: valida o arquivo inteiro e descreve o conteúdo.
use std::io::{self, Write};
use std::path::Path;

use crate::error::ArchiveError;
use crate::inspect::{report, InspectReport};
use crate::reader::ArchiveReader;

pub const EXIT_OK: u8 = 0;
pub const EXIT_ARCHIVE_ERROR: u8 = 2;
pub const EXIT_USAGE: u8 = 64;
pub const EXIT_IO: u8 = 74;

const USAGE: &str = "uso: tyrus-archive inspect [--json] <arquivo>";

/// # Errors
/// Falha de escrita em `stdout` ou `stderr`.
pub fn run(args: &[String], stdout: &mut impl Write, stderr: &mut impl Write) -> io::Result<u8> {
    let Some((json, file)) = parse(args) else {
        writeln!(stderr, "{USAGE}")?;
        return Ok(EXIT_USAGE);
    };
    match load(Path::new(&file)) {
        Ok(reader) => {
            render(&report(&reader), json, stdout)?;
            Ok(EXIT_OK)
        }
        Err(error) => {
            writeln!(stderr, "{}: {error}", error.stable_code())?;
            Ok(if matches!(error, ArchiveError::Io(_)) {
                EXIT_IO
            } else {
                EXIT_ARCHIVE_ERROR
            })
        }
    }
}

fn parse(args: &[String]) -> Option<(bool, String)> {
    let (command, rest) = args.split_first()?;
    if command != "inspect" {
        return None;
    }
    let json = rest.iter().any(|arg| arg == "--json");
    let mut files = rest.iter().filter(|arg| arg.as_str() != "--json");
    let file = files.next()?;
    files.next().is_none().then(|| (json, file.clone()))
}

fn load(path: &Path) -> Result<ArchiveReader, ArchiveError> {
    let reader = ArchiveReader::open(path)?;
    reader.verify()?;
    Ok(reader)
}

fn render(report: &InspectReport, json: bool, out: &mut impl Write) -> io::Result<()> {
    if json {
        serde_json::to_writer_pretty(&mut *out, report).map_err(io::Error::from)?;
        return writeln!(out);
    }
    writeln!(
        out,
        "formato {} | {} entradas | conjunto {}",
        report.format_version, report.entry_count, report.set_sha256
    )?;
    for entry in &report.entries {
        writeln!(
            out,
            "{:o} {:>10} {}  {}",
            entry.mode, entry.size, entry.sha256, entry.path
        )?;
    }
    Ok(())
}
