use std::fs::Metadata;
use std::path::{PathBuf, Path};
use std::io::ErrorKind;

use anyhow::Context;

pub fn get_metadata(path: &Path) -> Result<Option<Metadata>, std::io::Error> {
    match path.metadata() {
        Ok(m) => Ok(Some(m)),
        Err(err) => match err.kind() {
            ErrorKind::NotFound => Ok(None),
            _ => Err(err),
        }
    }
}

pub fn cwd() -> anyhow::Result<PathBuf> {
    std::env::current_dir()
        .context("failed to retrieve the current working directory")
}
