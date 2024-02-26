use std::path::{PathBuf, Path};
use std::io::ErrorKind;
use std::default::Default;
use std::io::{BufWriter, BufReader};

use serde::{Serialize, Deserialize};
use anyhow::Context;
use clap::Args;

pub mod tags;

#[derive(Debug, Args)]
pub struct FileList {
    /// the file(s) to act upon
    #[arg(trailing_var_arg = true, num_args(1..))]
    pub files: Vec<PathBuf>,
}

impl FileList {
    pub fn get_files(&self) -> anyhow::Result<FileListIter<'_>> {
        let cwd = std::env::current_dir()
            .context("failed to get current working directory")?;

        Ok(FileListIter {
            list_iter: self.files.iter(),
            cwd
        })
    }
}

#[derive(Debug)]
pub struct FileListIter<'a> {
    list_iter: std::slice::Iter<'a, PathBuf>,
    cwd: PathBuf,
}

impl<'a> FileListIter<'a> {
    fn get_file(&self, path: PathBuf) -> anyhow::Result<File> {
        let full_path = if !path.is_absolute() {
            let joined = self.cwd.join(&path);

            match joined.canonicalize() {
                Ok(canon) => canon,
                Err(_err) => joined
            }
        } else {
            path.clone()
        };

        let mut basename = full_path.file_name()
            .with_context(|| format!("given file has no basename: \"{}\"", full_path.display()))?
            .to_os_string();
        basename.push(".json");

        let mut meta_file = full_path.parent()
            .with_context(|| format!("given file has no parent directory: \"{}\"", full_path.display()))?
            .join(".file-meta");
        meta_file.push(basename);

        Ok(File::load(
            path,
            meta_file
        ).context("failed to load data for file")?)
    }
}

impl<'a> std::iter::Iterator for FileListIter<'a> {
    type Item = anyhow::Result<File>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(path) = self.list_iter.next() else {
            return None;
        };

        Some(self.get_file(path.clone()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileData {
    #[serde(default)]
    pub tags: tags::TagsMap,
    pub comment: Option<String>,
}

impl Default for FileData {
    fn default() -> Self {
        FileData {
            tags: tags::TagsMap::new(),
            comment: None
        }
    }
}

#[derive(Debug)]
pub struct File {
    ref_path: Box<Path>,
    path: Box<Path>,
    pub data: FileData,
}

impl File {
    pub fn load(ref_path: PathBuf, path: PathBuf) -> anyhow::Result<Self> {
        let reader = match std::fs::OpenOptions::new()
            .read(true)
            .open(&path) {
            Ok(f) => BufReader::new(f),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    return Ok(File {
                        ref_path: ref_path.into(),
                        path: path.into(),
                        data: Default::default(),
                    });
                }
                _ => {
                    return Err(err.into());
                }
            }
        };

        Ok(File {
            ref_path: ref_path.into(),
            path: path.into(),
            data: serde_json::from_reader(reader)?,
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let parent = self.path.parent()
            .context("missing parent directory name")?;

        if !parent.try_exists()? {
            std::fs::create_dir(&parent)
                .with_context(|| format!("failed creating parent directory for data file: \"{}\"", parent.display()))?;
        }

        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
            .with_context(|| format!("failed to open data file for writing: \"{}\"", self.path.display()))?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, &self.data)
            .with_context(|| format!("failed writing json data to data file: \"{}\"", self.path.display()))?;

        Ok(())
    }

    pub fn ref_path(&self) -> &Path {
        &self.ref_path
    }
}
