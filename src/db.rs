use std::collections::{BTreeMap, BTreeSet};
use std::default::Default;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::fs::get_metadata;
use crate::path;
use crate::tags;
use crate::time;

pub mod drop;
pub mod dump;
pub mod init;

#[derive(Debug, Args)]
pub struct DbArgs {
    #[command(subcommand)]
    cmd: ManageCmd,
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
    pub fn file_name(&self) -> &OsStr {
        match self {
            Format::JsonPretty => OsStr::new(DB_PRETTY_JSON_NAME),
            Format::Json => OsStr::new(DB_JSON_NAME),
            Format::Binary => OsStr::new(DB_BINARY_NAME),
        }
    }
}

pub const FORMAT_LIST: [Format; 3] = [Format::JsonPretty, Format::Json, Format::Binary];

pub trait MetaContainer: Debug {
    fn created(&self) -> &time::DateTime;
    fn updated(&self) -> Option<&time::DateTime>;
    fn modified(&self) -> &time::DateTime;

    fn tags(&self) -> &tags::TagsMap;
    fn comment(&self) -> Option<&str>;

    fn update_ts(&mut self);

    fn take_comment(&mut self) -> Option<String>;
    fn take_tags(&mut self) -> tags::TagsMap;
    fn take_tags_comment(&mut self) -> (tags::TagsMap, Option<String>);
}

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

impl MetaContainer for FileData {
    fn created(&self) -> &time::DateTime {
        &self.created
    }

    fn updated(&self) -> Option<&time::DateTime> {
        self.updated.as_ref()
    }

    fn modified(&self) -> &time::DateTime {
        self.updated.as_ref().unwrap_or(&self.created)
    }

    fn tags(&self) -> &tags::TagsMap {
        &self.tags
    }

    fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|v| v.as_str())
    }

    fn update_ts(&mut self) {
        self.updated = Some(time::datetime_now());
    }

    fn take_tags(&mut self) -> tags::TagsMap {
        std::mem::take(&mut self.tags)
    }

    fn take_comment(&mut self) -> Option<String> {
        std::mem::take(&mut self.comment)
    }

    fn take_tags_comment(&mut self) -> (tags::TagsMap, Option<String>) {
        (
            std::mem::take(&mut self.tags),
            std::mem::take(&mut self.comment),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Db {
    pub files: BTreeMap<Box<str>, FileData>,
    pub collections: BTreeMap<String, BTreeSet<Box<str>>>,
    pub tags: tags::TagsMap,
    pub comment: Option<String>,
    #[serde(default = "time::datetime_now")]
    pub created: time::DateTime,
    pub updated: Option<time::DateTime>,
}

impl Default for Db {
    fn default() -> Self {
        Db {
            files: BTreeMap::new(),
            collections: BTreeMap::new(),
            tags: tags::TagsMap::new(),
            comment: None,
            created: time::datetime_now(),
            updated: None,
        }
    }
}

impl MetaContainer for Db {
    fn created(&self) -> &time::DateTime {
        &self.created
    }

    fn updated(&self) -> Option<&time::DateTime> {
        self.updated.as_ref()
    }

    fn modified(&self) -> &time::DateTime {
        self.updated.as_ref().unwrap_or(&self.created)
    }

    fn tags(&self) -> &tags::TagsMap {
        &self.tags
    }

    fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|v| v.as_str())
    }

    fn update_ts(&mut self) {
        self.updated = Some(time::datetime_now());
    }

    fn take_tags(&mut self) -> tags::TagsMap {
        std::mem::take(&mut self.tags)
    }

    fn take_comment(&mut self) -> Option<String> {
        std::mem::take(&mut self.comment)
    }

    fn take_tags_comment(&mut self) -> (tags::TagsMap, Option<String>) {
        (
            std::mem::take(&mut self.tags),
            std::mem::take(&mut self.comment),
        )
    }
}

#[derive(Debug)]
pub struct Context {
    format: Format,
    pub db: Db,
    path: DbPath,
    root: RootPath,
}

impl Context {
    pub fn create<P>(path: P, format: Format) -> anyhow::Result<Self>
    where
        P: Into<DbPath>,
    {
        let path = path.into();
        let root = Self::get_root(&path);

        let rtn = Context {
            format,
            db: Db::default(),
            path,
            root,
        };

        rtn.write_file(true)?;

        Ok(rtn)
    }

    fn get_root(path: &Path) -> RootPath {
        path.parent().unwrap().parent().unwrap().into()
    }

    pub fn find_file<P>(ref_path: P) -> anyhow::Result<Option<(DbPath, Format)>>
    where
        P: AsRef<Path>,
    {
        let ref_path = ref_path.as_ref();

        for ancestor in ref_path.ancestors() {
            let fsm_dir = ancestor.join(".fsm");

            let Some(metadata) =
                get_metadata(&fsm_dir).context("io error when checkign for .fsm directory")?
            else {
                continue;
            };

            if !metadata.is_dir() {
                continue;
            }

            for format in &FORMAT_LIST {
                let db_file = fsm_dir.join(format.file_name());

                let Some(metadata) =
                    get_metadata(&db_file).context("io error when checking for db file")?
                else {
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

    fn read_file(path: Box<Path>, format: Format) -> anyhow::Result<Self> {
        log::info!("reading {}", path.display());

        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .with_context(|| format!("failed reading db: {}", path.display()))?;
        let reader = BufReader::new(file);

        let start = std::time::Instant::now();

        let db = match &format {
            Format::JsonPretty | Format::Json => serde_json::from_reader(reader)
                .with_context(|| format!("failed deserializing db json: {}", path.display()))?,
            Format::Binary => bincode::deserialize_from(reader)
                .with_context(|| format!("failed deserializing db binary: {}", path.display()))?,
        };

        log::info!("db parse time: {:?}", start.elapsed());

        let root = Self::get_root(&path);

        Ok(Context {
            format,
            db,
            path,
            root,
        })
    }

    pub fn cwd_load() -> anyhow::Result<Self> {
        let Some((path, format)) = Self::find_file(path::get_cwd())? else {
            return Err(anyhow::anyhow!("no db found"));
        };

        Self::read_file(path, format)
    }

    fn write_file(&self, create: bool) -> anyhow::Result<()> {
        if create {
            log::info!("creating {}", self.path.display());
        } else {
            log::info!("writing {}", self.path.display());
        }

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(create)
            .open(&self.path)
            .with_context(|| format!("failed to open db file: {}", self.path.display()))?;
        let writer = BufWriter::new(file);

        let start = std::time::Instant::now();

        match &self.format {
            Format::JsonPretty => serde_json::to_writer_pretty(writer, &self.db)
                .with_context(|| format!("failed serializing db json: {}", self.path.display()))?,
            Format::Json => serde_json::to_writer(writer, &self.db)
                .with_context(|| format!("failed serializing db json: {}", self.path.display()))?,
            Format::Binary => bincode::serialize_into(writer, &self.db).with_context(|| {
                format!("failed serializing db binary: {}", self.path.display())
            })?,
        }

        log::info!("db save time: {:?}", start.elapsed());

        Ok(())
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
