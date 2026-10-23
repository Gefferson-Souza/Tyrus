//! Argumentos de `tyrus-oracle record`.
use std::path::PathBuf;

pub const USAGE: &str =
    "uso: tyrus-oracle record --corpus <nome|diretório> [--check] [--lock <caminho>] \
[--front <cli.ts>] [--emit-dir <diretório>] [--no-emit] [--seed-results <diretório>]";

const CORPUS_ROOT: &str = "oracle/corpus";
const DEFAULT_LOCK: &str = "oracle/oracle.lock";
const DEFAULT_FRONT: &str = "front/src/cli.ts";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordOptions {
    pub corpus: PathBuf,
    pub check: bool,
    pub lock: PathBuf,
    pub front: PathBuf,
    pub emit_dir: Option<PathBuf>,
    pub no_emit: bool,
    pub seed_results: Option<PathBuf>,
}

fn resolve_corpus(value: &str) -> PathBuf {
    if value.contains('/') {
        PathBuf::from(value)
    } else {
        PathBuf::from(CORPUS_ROOT).join(value)
    }
}

impl RecordOptions {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self {
            corpus: PathBuf::new(),
            check: false,
            lock: PathBuf::from(DEFAULT_LOCK),
            front: PathBuf::from(DEFAULT_FRONT),
            emit_dir: None,
            no_emit: false,
            seed_results: None,
        };
        let mut rest = args.iter();
        while let Some(flag) = rest.next() {
            match flag.as_str() {
                "--check" => options.check = true,
                "--no-emit" => options.no_emit = true,
                other => {
                    let value = rest
                        .next()
                        .ok_or_else(|| format!("{other} exige um valor"))?;
                    options.set_valued(other, value)?;
                }
            }
        }
        if options.corpus.as_os_str().is_empty() {
            return Err("--corpus é obrigatório".to_owned());
        }
        Ok(options)
    }

    fn set_valued(&mut self, flag: &str, value: &str) -> Result<(), String> {
        match flag {
            "--corpus" => self.corpus = resolve_corpus(value),
            "--lock" => self.lock = PathBuf::from(value),
            "--front" => self.front = PathBuf::from(value),
            "--emit-dir" => self.emit_dir = Some(PathBuf::from(value)),
            "--seed-results" => self.seed_results = Some(PathBuf::from(value)),
            other => return Err(format!("opção desconhecida `{other}`")),
        }
        Ok(())
    }

    /// Pasta de saída do sidecar; o JS emitido fica em `<pasta>/emit`.
    #[must_use]
    pub fn out_dir(&self) -> PathBuf {
        self.emit_dir
            .clone()
            .unwrap_or_else(|| self.corpus.join("dist"))
    }
}
