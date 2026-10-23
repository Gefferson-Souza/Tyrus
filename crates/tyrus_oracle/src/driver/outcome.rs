//! Resultado bruto de uma execução de sonda.
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// `None` quando o processo morreu por sinal (inclui o abate por tempo esgotado).
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub timed_out: bool,
}

impl Outcome {
    #[must_use]
    pub fn same_observable(&self, other: &Self) -> bool {
        self.exit_code == other.exit_code
            && self.stdout == other.stdout
            && self.stderr == other.stderr
    }

    /// Primeira linha `Name: message` do erro não capturado, quando houver.
    #[must_use]
    pub fn error_first_line(&self) -> Option<String> {
        let text = String::from_utf8_lossy(&self.stderr);
        text.lines()
            .map(str::trim)
            .find(|line| is_error_line(line))
            .map(str::to_owned)
    }
}

fn is_error_line(line: &str) -> bool {
    let Some((name, _)) = line.split_once(": ") else {
        return false;
    };
    name.ends_with("Error") || name.ends_with("Exception")
}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        for nibble in [byte >> 4, byte & 0x0f] {
            text.extend(char::from_digit(u32::from(nibble), 16));
        }
    }
    text
}
