use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::io::{ErrorKind, BufWriter, BufReader};
use std::default::Default;
use std::ffi::OsStr;
use std::fs::OpenOptions;

use serde::{Serialize, Deserialize};
use anyhow::Context;
use clap::ValueEnum;

use crate::fs::{cwd, get_metadata};
use crate::tags;

type DbPath = Box<Path>;
type FilePath = Box<Path>;
type RootPath = Box<Path>;

const DB_PRETTY_JSON_NAME: &str = "db.pretty.json";
const DB_JSON_NAME: &str = "db.json";
const DB_BINARY_NAME: &str = "db.bincode";

#[derive(Debug, Clone, ValueEnum)]
pub enum FileType {
    JsonPretty,
    Json,
    Binary,
}

impl FileType {
    pub fn get_file_name_os(&self) -> &OsStr {
        match self {
            FileType::JsonPretty => OsStr::new(DB_PRETTY_JSON_NAME),
            FileType::Json => OsStr::new(DB_JSON_NAME),
            FileType::Binary => OsStr::new(DB_BINARY_NAME),
        }
    }
}

pub const DB_TYPE_LIST: [FileType; 3] = [
    FileType::JsonPretty,
    FileType::Json,
    FileType::Binary,
];

#[derive(Debug, Serialize, Deserialize)]
pub struct FileData {
    pub tags: tags::TagsMap,
    pub comment: Option<String>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for FileData {
    fn default() -> Self {
        FileData {
            tags: tags::TagsMap::new(),
            comment: None,
            created: chrono::Utc::now(),
            updated: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inner {
    pub files: BTreeMap<FilePath, FileData>,
    pub collections: BTreeMap<String, BTreeSet<FilePath>>,
}

impl Default for Inner {
    fn default() -> Self {
        Inner {
            files: BTreeMap::new(),
            collections: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct DbLock {
    path: FilePath,
}

impl DbLock {
    fn check_exists(dir: &Path) -> anyhow::Result<bool> {
        let lock_file = dir.join("db.lock");

        let Some(metadata) = get_metadata(&lock_file)
            .context("failed to get metadata for db.lock")? else {
            return Ok(false);
        };

        if metadata.is_file() {
            Ok(true)
        } else {
            Err(anyhow::anyhow!("a db.lock exists but is not a file"))
        }
    }

    fn create(dir: &Path) -> anyhow::Result<Self> {
        let path = dir.join("db.lock");

        let open_result = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path);

        if let Err(err) = open_result {
            match err.kind() {
                ErrorKind::AlreadyExists => {
                    return Err(anyhow::anyhow!("a db.lock already exists."));
                },
                _ => {
                    return Err(anyhow::Error::new(err)
                        .context("failed to create db.lock"));
                }
            }
        }

        Ok(DbLock { path: path.into() })
    }

    fn drop(self) -> anyhow::Result<()> {
        std::fs::remove_file(self.path)
            .context("failed to remove db.lock")
    }
}

#[derive(Debug)]
pub struct Db {
    file_type: FileType,
    pub inner: Inner,
    path: DbPath,
    root: RootPath,
}

impl Db {
    pub fn new<P>(path: P, file_type: FileType) -> Self
    where
        P: Into<DbPath>,
    {
        let path = path.into();
        let root = Self::get_root(&path);

        Db {
            file_type,
            inner: Inner::default(),
            path,
            root,
        }
    }

    fn get_root(path: &Path) -> RootPath {
        path.parent()
            .unwrap()
            .parent()
            .unwrap()
            .into()
    }

    pub fn find_file<P>(ref_path: P) -> anyhow::Result<Option<(DbPath, FileType)>>
    where
        P: AsRef<Path>
    {
        let ref_path = ref_path.as_ref();

        for ancestor in ref_path.ancestors() {
            let fsm_dir = ancestor.join(".fsm");

            let Some(metadata) = get_metadata(&fsm_dir)
                .context("io error when checkign for .fsm directory")? else {
                continue;
            };

            if !metadata.is_dir() {
                continue;
            }

            for file_type in &DB_TYPE_LIST {
                let db_file = fsm_dir.join(file_type.get_file_name_os());

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

    fn read_file<P>(path: P, file_type: FileType) -> anyhow::Result<Self>
    where
        P: Into<Box<Path>>
    {
        let path = path.into();

        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .with_context(|| format!("failed reading db: {}", path.display()))?;
        let reader = BufReader::new(file);

        let start = std::time::Instant::now();

        let inner = match &file_type {
            FileType::JsonPretty |
            FileType::Json => serde_json::from_reader(reader)
                .with_context(|| format!("failed deserializing db json: {}", path.display()))?,
            FileType::Binary => bincode::deserialize_from(reader)
                .with_context(|| format!("failed deserializing db binary: {}", path.display()))?
        };

        log::info!("db parse time: {:?}", start.elapsed());

        let root = Self::get_root(&path);

        log::debug!("loaded {}", path.display());

        Ok(Db {
            file_type,
            inner,
            path,
            root,
        })
    }

    pub fn load<P>(path: P, file_type: FileType) -> anyhow::Result<Self>
    where
        P: Into<Box<Path>>
    {
        Self::read_file(path, file_type)
    }

    pub fn cwd_load() -> anyhow::Result<Self> {
        let cwd = cwd()?;

        let Some((path, file_type)) = Self::find_file(&cwd)? else {
            return Err(anyhow::anyhow!("no db found"));
        };

        Self::load(path, file_type)
    }

    fn write_file(&self, create: bool) -> anyhow::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(create)
            .open(&self.path)
            .with_context(|| format!("failed to open db file: {}", self.path.display()))?;
        let writer = BufWriter::new(file);

        let start = std::time::Instant::now();

        match &self.file_type {
            FileType::JsonPretty => serde_json::to_writer_pretty(writer, &self.inner)
                .with_context(|| format!("failed serializing db json: {}", self.path.display()))?,
            FileType::Json => serde_json::to_writer(writer, &self.inner)
                .with_context(|| format!("failed serializing db json: {}", self.path.display()))?,
            FileType::Binary => bincode::serialize_into(writer, &self.inner)
                .with_context(|| format!("failed serializing db binary: {}", self.path.display()))?
        }

        log::info!("db save time: {:?}", start.elapsed());

        Ok(())
    }

    pub fn create(&self) -> anyhow::Result<()> {
        self.write_file(true)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.write_file(false)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn common_root<P>(&self, ref_path: P) -> anyhow::Result<FilePath>
    where
        P: AsRef<Path>
    {
        let ref_path = ref_path.as_ref();

        log::debug!("root: {} ref_path: {}", self.root.display(), ref_path.display());

        let from_root = ref_path.strip_prefix(&self.root)
            .with_context(|| format!("file and db do not share a common root {}", ref_path.display()))?;

        Ok(from_root.into())
    }

    pub fn maybe_common_root<P>(&self, ref_path: P) -> Option<FilePath>
    where
        P: AsRef<Path>
    {
        match self.common_root(ref_path) {
            Ok(p) => Some(p),
            Err(err) => {
                println!("{}", err);
                None
            }
        }
    }
}
