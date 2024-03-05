use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Context;
use path_absolutize::Absolutize as _;

static CWD: OnceLock<Box<Path>> = OnceLock::new();

pub fn set_cwd() -> anyhow::Result<()> {
    let result = std::env::current_dir()
        .context("failed to retrieve current working directory")?;

    let _ = CWD.set(result.into());

    Ok(())
}

pub fn get_cwd() -> &'static Path {
    CWD.get().unwrap()
}

pub struct RelativePath {
    full: Box<Path>,
    to_db: Option<Box<Path>>,
}

impl RelativePath {
    pub fn from_root(root: &Path, given: &PathBuf) -> anyhow::Result<Self> {
        let rtn = if !given.is_absolute() {
            given.absolutize_from(get_cwd())
                .with_context(|| format!("failed to resolve path: {}", given.display()))?
                .into()
        } else {
            given.clone()
        };

        let to_db = rtn.strip_prefix(root)
            .ok()
            .map(|v| v.into());

        Ok(RelativePath {
            full: rtn.into(),
            to_db
        })

    }

    pub fn full_path(&self) -> &Path {
        &self.full
    }

    pub fn to_db(&self) -> anyhow::Result<&Path> {
        if let Some(valid) = &self.to_db {
            Ok(valid)
        } else {
            Err(anyhow::anyhow!("file and db do not share a common root: {}", self.full.display()))
        }
    }

    pub fn display(&self) -> std::path::Display<'_> {
        self.full.display()
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
            root
        }
    }
}

impl<'a> std::iter::Iterator for RelativePathList<'a> {
    type Item = anyhow::Result<RelativePath>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(path) = self.iter.next() else {
            return None;
        };

        Some(RelativePath::from_root(&self.root, path))
    }
}
