#![allow(dead_code)]
use std::fs;
use std::path::{Path, PathBuf};

use tyrus_facts::{sha256_hex, FileFact, Sha256Hex};

pub(crate) const REPLICA_FACTS: &str = include_str!("../golden/replica.facts.json");

/// Diretório temporário único por teste, removido ao sair de escopo.
pub(crate) struct TempTree {
    root: PathBuf,
}

impl TempTree {
    pub(crate) fn new(label: &str) -> Self {
        let base =
            std::env::var_os("CARGO_TARGET_TMPDIR").map_or_else(std::env::temp_dir, PathBuf::from);
        let root = base.join(format!("tyrus-facts-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("criar diretório temporário");
        Self { root }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.root
    }

    pub(crate) fn write(&self, relative: &str, content: &[u8]) -> FileFact {
        let target = self.root.join(relative);
        fs::create_dir_all(target.parent().expect("pai")).expect("criar pais");
        fs::write(&target, content).expect("escrever arquivo");
        FileFact::new(relative, sha256_hex(content)).expect("fato válido")
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub(crate) fn digest_of(text: &str) -> Sha256Hex {
    sha256_hex(text.as_bytes())
}
