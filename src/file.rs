use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::io::ErrorKind;
use std::default::Default;
use std::io::{BufWriter, BufReader};
use std::fmt::{Display, Formatter};

use serde::{Serialize, Deserialize};
use url::Url;
use anyhow::Context;
use clap::Args;

#[derive(Debug, Args)]
pub struct FileList {
    /// the file(s) to act upon
    #[arg(trailing_var_arg = true, num_args(1..))]
    pub files: Vec<PathBuf>,
}

impl FileList {
    pub fn get_files(&self) -> anyhow::Result<Vec<File>> {
        let mut files = Vec::with_capacity(self.files.len());
        let cwd = std::env::current_dir()?;

        for path in &self.files {
            let full_path = if !path.is_absolute() {
                let joined = cwd.join(&path);

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

            files.push(File::load(path.clone(), meta_file).context("failed to load data for file")?);
        }

        Ok(files)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagValue {
    Simple(String),
    Number(i64),
    Url(url::Url),
}

impl TagValue {
    fn new(value: String) -> Self {
        TagValue::Simple(value)
    }

    fn parse_url(value: &str) -> Result<Self, url::ParseError> {
        Ok(TagValue::Url(Url::parse(value)?))
    }

    fn parse_num(value: &str) -> anyhow::Result<Self, std::num::ParseIntError> {
        Ok(TagValue::Number(value.parse()?))
    }
}

impl Display for TagValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TagValue::Simple(v) => write!(f, "{}", v),
            TagValue::Number(v) => write!(f, "{}", v),
            TagValue::Url(v) => write!(f, "{}", v),
        }
    }
}

impl From<&str> for TagValue {
    fn from(value: &str) -> Self {
        if let Ok(i64_value) = value.parse() {
            TagValue::Number(i64_value)
        } else if let Ok(url) = value.parse() {
            TagValue::Url(url)
        } else {
            TagValue::Simple(value.to_owned())
        }
    }
}

pub type Tag = (String, Option<TagValue>);

pub fn parse_tag(arg: &str) -> Result<Tag, String> {
    if let Some((name, value)) = arg.split_once(':') {
        if name.is_empty() {
            return Err(format!("tag name is empty: \"{}\"", arg));
        }

        if value.is_empty() {
            Ok((name.into(), None))
        } else {
            Ok((name.into(), Some(value.into())))
        }
    } else {
        if arg.is_empty() {
            return Err(format!("tag is empty: \"{}\"", arg));
        }

        Ok((arg.into(), None))
    }
}

pub fn parse_url_tag(arg: &str) -> Result<Tag, String> {
    if let Some((name, value)) = arg.split_once(':') {
        if name.is_empty() {
            return Err(format!("tag name is empty: \"{}\"", arg));
        }

        if value.is_empty() {
            return Err(format!("missing url data: \"{}\"", arg));
        }

        match TagValue::parse_url(value) {
            Ok(url) => Ok((name.into(), Some(url))),
            Err(err) => {
                Err(format!("invalid url provided: \"{}\" {}", value, err))
            }
        }
    } else {
        Err(format!("missing tag value: \"{}\"", arg))
    }
}

pub fn parse_num_tag(arg: &str) -> Result<Tag, String> {
    if let Some((name, value)) = arg.split_once(':') {
        if name.is_empty() {
            return Err(format!("tag name is empty: \"{}\"", arg));
        }

        if value.is_empty() {
            return Err(format!("missing num data: \"{}\"", arg));
        }

        match TagValue::parse_num(value) {
            Ok(url) => Ok((name.into(), Some(url))),
            Err(err) => {
                Err(format!("invalid num provided: \"{}\" {}", value, err))
            }
        }
    } else {
        Err(format!("missing tag value: \"{}\"", arg))
    }
}

pub type TagsMap = HashMap<String, Option<TagValue>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileData {
    #[serde(default)]
    pub tags: TagsMap,
    pub comment: Option<String>,
}

impl Default for FileData {
    fn default() -> Self {
        FileData {
            tags: HashMap::new(),
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
