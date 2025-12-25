use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Context;
use path_absolutize::Absolutize as _;
use thiserror::Error;

static CWD: OnceLock<Box<Path>> = OnceLock::new();

pub fn set_cwd() -> anyhow::Result<()> {
    let result = std::env::current_dir().context("failed to retrieve current working directory")?;

    let _ = CWD.set(result.into());

    Ok(())
}

pub fn get_cwd() -> &'static Path {
    CWD.get().unwrap()
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("file and db do not share a common root: {}", .0.display())]
    InvalidPrefix(PathBuf),

    #[error("file path contains non-UTF-8 characters: {}", .0.display())]
    InvalidChars(PathBuf),

    #[error("io error when resolving path {}", .1.display())]
    Io(std::io::Error, PathBuf),
}

pub struct RelativePath {
    full: Box<Path>,
    db_entry: Box<str>,
}

impl RelativePath {
    pub fn from_root(root: &Path, given: &PathBuf) -> Result<Self, PathError> {
        let rtn = if !given.is_absolute() {
            match given.absolutize_from(get_cwd()) {
                Ok(v) => v.into(),
                Err(err) => {
                    return Err(PathError::Io(err, given.clone()));
                }
            }
        } else {
            given.clone()
        };

        let Ok(from_root) = rtn.strip_prefix(root) else {
            return Err(PathError::InvalidPrefix(rtn.clone()));
        };

        let Some(utf_from_root) = from_root.to_str() else {
            return Err(PathError::InvalidChars(rtn.clone()));
        };

        let db_entry = if std::path::MAIN_SEPARATOR != '/' {
            utf_from_root
                .replace(std::path::MAIN_SEPARATOR_STR, "/")
                .into()
        } else {
            utf_from_root.into()
        };

        Ok(RelativePath {
            full: rtn.into(),
            db_entry,
        })
    }

    pub fn full_path(&self) -> &Path {
        &self.full
    }

    pub fn db_entry(&self) -> &str {
        &self.db_entry
    }

    pub fn display(&self) -> std::path::Display<'_> {
        self.full.display()
    }
}

impl From<RelativePath> for (Box<Path>, Box<str>) {
    fn from(rel_path: RelativePath) -> Self {
        (rel_path.full, rel_path.db_entry)
    }
}

pub struct RelativePathList<'a> {
    iter: std::slice::Iter<'a, PathBuf>,
    root: Box<Path>,
}

impl<'a> RelativePathList<'a> {
    pub fn new(root: Box<Path>, path_list: &'a Vec<PathBuf>) -> Self {
        RelativePathList {
            iter: path_list.iter(),
            root,
        }
    }
}

impl<'a> std::iter::Iterator for RelativePathList<'a> {
    type Item = Result<RelativePath, PathError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(path) = self.iter.next() else {
            return None;
        };

        Some(RelativePath::from_root(&self.root, path))
    }
}
