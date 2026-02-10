use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FilePath(pub PathBuf);

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl From<PathBuf> for FilePath {
    fn from(path: PathBuf) -> Self {
        FilePath(path)
    }
}

impl From<&str> for FilePath {
    fn from(path: &str) -> Self {
        FilePath(PathBuf::from(path))
    }
}
