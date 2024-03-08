use std::collections::{BTreeMap, BTreeSet};
use std::path::{PathBuf, Path};
use std::io::{BufWriter, BufReader};
use std::default::Default;
use std::ffi::OsStr;
use std::fs::OpenOptions;

use serde::{Serialize, Deserialize};
use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};

use crate::fs::get_metadata;
use crate::tags;
use crate::path;
use crate::time;

pub mod init;
pub mod dump;
pub mod drop;

#[derive(Debug, Args)]
pub struct DbArgs {
    #[command(subcommand)]
    cmd: ManageCmd
}

#[derive(Debug, Subcommand)]
enum ManageCmd {
    /// initializes a directory with an fsm db
    Init(init::InitArgs),

    /// dumps a database file to stdout
    Dump(dump::DumpArgs),

    /// drops a db and fsm directory
    Drop(drop::DropArgs),
}

pub fn manage(args: DbArgs) -> anyhow::Result<()> {
    match args.cmd {
        ManageCmd::Init(init_args) => init::init_db(init_args),
        ManageCmd::Dump(dump_args) => dump::dump_db(dump_args),
        ManageCmd::Drop(drop_args) => drop::drop_db(drop_args),
    }
}

type DbPath = Box<Path>;
type RootPath = Box<Path>;

const DB_PRETTY_JSON_NAME: &str = "db.pretty.json";
const DB_JSON_NAME: &str = "db.json";
const DB_BINARY_NAME: &str = "db.bincode";

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    JsonPretty,
    Json,
    Binary,
}

impl Format {
    pub fn get_file_name_os(&self) -> &OsStr {
        match self {
            Format::JsonPretty => OsStr::new(DB_PRETTY_JSON_NAME),
            Format::Json => OsStr::new(DB_JSON_NAME),
            Format::Binary => OsStr::new(DB_BINARY_NAME),
        }
    }
}

pub const FORMAT_LIST: [Format; 3] = [
    Format::JsonPretty,
    Format::Json,
    Format::Binary,
];

#[derive(Debug, Serialize, Deserialize)]
pub struct FileData {
    pub tags: tags::TagsMap,
    pub comment: Option<String>,
    pub created: time::DateTime,
    pub updated: Option<time::DateTime>,
}

impl Default for FileData {
    fn default() -> Self {
        FileData {
            tags: tags::TagsMap::new(),
            comment: None,
            created: time::datetime_now(),
            updated: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inner {
    pub files: BTreeMap<Box<str>, FileData>,
    pub collections: BTreeMap<String, BTreeSet<Box<str>>>,
    pub tags: tags::TagsMap,
    pub comment: Option<String>,
}

impl Default for Inner {
    fn default() -> Self {
        Inner {
            files: BTreeMap::new(),
            collections: BTreeMap::new(),
            tags: tags::TagsMap::new(),
            comment: None,
        }
    }
}

#[derive(Debug)]
pub struct Db {
    format: Format,
    pub inner: Inner,
    path: DbPath,
    root: RootPath,
}

impl Db {
    pub fn new<P>(path: P, format: Format) -> Self
    where
        P: Into<DbPath>,
    {
        let path = path.into();
        let root = Self::get_root(&path);

        Db {
            format,
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

    pub fn find_file<P>(ref_path: P) -> anyhow::Result<Option<(DbPath, Format)>>
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

            for format in &FORMAT_LIST {
                let db_file = fsm_dir.join(format.get_file_name_os());

                let Some(metadata) = get_metadata(&db_file)
                    .context("io error when checking for db file")? else {
                    continue;
                };

                if !metadata.is_file() {
                    continue;
                }

                return Ok(Some((db_file.into(), format.clone())));
            }
        }

        Ok(None)
    }

    fn read_file<P>(path: P, format: Format) -> anyhow::Result<Self>
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

        let inner = match &format {
            Format::JsonPretty |
            Format::Json => serde_json::from_reader(reader)
                .with_context(|| format!("failed deserializing db json: {}", path.display()))?,
            Format::Binary => bincode::deserialize_from(reader)
                .with_context(|| format!("failed deserializing db binary: {}", path.display()))?
        };

        log::info!("db parse time: {:?}", start.elapsed());

        let root = Self::get_root(&path);

        log::debug!("loaded {}", path.display());

        Ok(Db {
            format,
            inner,
            path,
            root,
        })
    }

    pub fn load<P>(path: P, format: Format) -> anyhow::Result<Self>
    where
        P: Into<Box<Path>>
    {
        Self::read_file(path, format)
    }

    pub fn cwd_load() -> anyhow::Result<Self> {
        let Some((path, format)) = Self::find_file(path::get_cwd())? else {
            return Err(anyhow::anyhow!("no db found"));
        };

        Self::load(path, format)
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

        match &self.format {
            Format::JsonPretty => serde_json::to_writer_pretty(writer, &self.inner)
                .with_context(|| format!("failed serializing db json: {}", self.path.display()))?,
            Format::Json => serde_json::to_writer(writer, &self.inner)
                .with_context(|| format!("failed serializing db json: {}", self.path.display()))?,
            Format::Binary => bincode::serialize_into(writer, &self.inner)
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

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn root_copy(&self) -> Box<Path> {
        self.root.clone()
    }

    pub fn rel_to_db(&self, path: PathBuf) -> Result<path::RelativePath, path::PathError> {
        path::RelativePath::from_root(&self.root, &path)
    }

    pub fn rel_to_db_list<'a>(&self, path_list: &'a Vec<PathBuf>) -> path::RelativePathList<'a> {
        path::RelativePathList::new(self.root.clone(), path_list)
    }
}
