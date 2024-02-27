use std::collections::{HashMap, BTreeMap};
use std::path::{Path, PathBuf};
use std::io::{ErrorKind, BufWriter, BufReader};
use std::fs::Metadata;

use serde::{Serialize, Deserialize};
use anyhow::Context;

use crate::file::tags;

type DbPath = Box<Path>;
type FilePath = Box<Path>;

#[derive(Debug, Clone)]
pub enum FileType {
    JsonPretty,
    Json,
    Binary
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileData {
    pub tags: tags::TagsMap,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inner {
    pub files: BTreeMap<FilePath, FileData>,
}

#[derive(Debug)]
pub struct DbLock {
    path: FilePath,
}

#[derive(Debug)]
pub struct Db {
    file_type: FileType,
    inner: Inner,
    path: DbPath,
}

fn get_metadata(path: &Path) -> Result<Option<Metadata>, std::io::Error> {
    match path.metadata() {
        Ok(m) => Ok(Some(m)),
        Err(err) => match err.kind() {
            ErrorKind::NotFound => Ok(None),
            _ => Err(err),
        }
    }
}

impl Db {
    pub fn find_file(ref_path: &PathBuf) -> anyhow::Result<Option<(DbPath, FileType)>> {
        let to_check = [
            ("db.pretty.json", FileType::JsonPretty),
            ("db.json", FileType::Json),
            ("db.bincode", FileType::Binary),
        ];

        for ancestor in ref_path.ancestors() {
            let fsm_dir = ancestor.join(".fsm");

            let Some(metadata) = get_metadata(&fsm_dir)
                .context("io error when checkign for .fsm directory")? else {
                continue;
            };

            if !metadata.is_dir() {
                continue;
            }

            for (check, file_type) in &to_check {
                let db_file = fsm_dir.join(check);

                let Some(metadata) = get_metadata(&db_file)
                    .context("io error when checking for db file")? else {
                    continue;
                };

                if !metadata.is_file() {
                    continue;
                }

                return Ok(Some((db_file.into(), file_type.clone())));
            }
        }

        Ok(None)
    }

    pub fn load(path: PathBuf, file_type: FileType) -> anyhow::Result<Option<Self>> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(&path)
            .with_context(|| format!("failed reading db: {}", path.display()))?;
        let reader = BufReader::new(file);

        let inner = match &file_type {
            FileType::JsonPretty |
            FileType::Json => serde_json::from_reader(reader)
                .with_context(|| format!("failed deserializing db json: {}", path.display()))?,
            FileType::Binary => bincode::deserialize_from(reader)
                .with_context(|| format!("failed deserializing db binary: {}", path.display()))?
        };

        Ok(Some(Db {
            file_type: file_type,
            inner,
            path: path.into(),
        }))
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .with_context(|| format!("failed to open db file: {}", self.path.display()))?;
        let writer = BufWriter::new(file);

        match &self.file_type {
            FileType::JsonPretty => serde_json::to_writer_pretty(writer, &self.inner)
                .with_context(|| format!("failed serializing db json: {}", self.path.display()))?,
            FileType::Json => serde_json::to_writer(writer, &self.inner)
                .with_context(|| format!("failed serializing db json: {}", self.path.display()))?,
            FileType::Binary => bincode::serialize_into(writer, &self.inner)
                .with_context(|| format!("failed serializing db binary: {}", self.path.display()))?
        }

        Ok(())
    }
}

struct WorkingSet {
    dbs: HashMap<DbPath, Db>,
    files: HashMap<FilePath, DbPath>,
}

impl WorkingSet {
    pub fn add_file(&mut self, ref_path: PathBuf) -> anyhow::Result<()> {
        let Some((path, file_type)) = Db::find_file(&ref_path)? else {
            return Err(anyhow::anyhow!("Failed to find db file: {}", ref_path.display()));
        };

        Ok(())
    }
}
