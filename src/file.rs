use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use path_absolutize::Absolutize as _;

pub fn log_path_result(path_result: anyhow::Result<PathBuf>) -> Option<PathBuf> {
    match path_result {
        Ok(p) => Some(p),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}

#[derive(Debug, Args)]
pub struct FileList {
    /// the file(s) to act upon
    #[arg(trailing_var_arg = true, num_args(1..))]
    pub files: Vec<PathBuf>,
}

impl FileList {
    pub fn get_canon(&self) -> anyhow::Result<CanonFiles<'_>> {
        let cwd = std::env::current_dir()
            .context("failed to get current working directory")?;

        Ok(CanonFiles {
            list_iter: self.files.iter(),
            cwd,
        })
    }
}

#[derive(Debug)]
pub struct CanonFiles<'a> {
    list_iter: std::slice::Iter<'a, PathBuf>,
    cwd: PathBuf,
}

impl<'a> CanonFiles<'a> {
    pub fn new(list: &'a Vec<PathBuf>) -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()
            .context("failed to get current working directory")?;

        Ok(CanonFiles {
            list_iter: list.iter(),
            cwd,
        })
    }

    fn get_path(&self, path: &PathBuf) -> anyhow::Result<PathBuf> {
        let rtn = if !path.is_absolute() {
            let resolved = path.absolutize_from(&self.cwd)
                .with_context(|| format!("failed to canonicalize path: {}", path.display()))?;

            resolved.into()
        } else {
            path.clone()
        };

        Ok(rtn)
    }
}

impl<'a> std::iter::Iterator for CanonFiles<'a> {
    type Item = anyhow::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(path) = self.list_iter.next() else {
            return None;
        };

        Some(self.get_path(path))
    }
}
