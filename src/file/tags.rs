use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use serde::{Serialize, Deserialize};
use clap::Args;
use url::Url;

#[derive(Debug, Args)]
pub struct TagArgs {
    /// set a tag to the files
    ///
    /// this will override all previously set tags for the files to only
    /// include the provided tags
    #[arg(
        short,
        long,
        conflicts_with_all(["add_tag", "drop_tag"]),
        value_parser(parse_tag)
    )]
    pub tag: Vec<Tag>,

    /// set a url tag to the files
    ///
    /// similar to a regular tag but if the tag value is not a valid url then
    /// the operation will fail
    #[arg(
        long,
        conflicts_with_all(["add", "add_url", "add_num", "drop"]),
        value_parser(parse_url_tag)
    )]
    pub tag_url: Vec<Tag>,

    /// set a number tag to the files
    ///
    /// similar to the regular tag but if the tag value is not a valid integer
    /// then the operation will fail
    #[arg(
        long,
        conflicts_with_all(["add", "add_url", "add_num", "drop"]),
        value_parser(parse_num_tag)
    )]
    pub tag_num: Vec<Tag>,

    /// add a tag to the files
    ///
    /// this will add to the existing list of tags for the specified files
    #[arg(
        short = 'a',
        long,
        conflicts_with_all(["tag", "tag_url", "tag_num"]),
        value_parser(parse_tag)
    )]
    pub add: Vec<Tag>,

    /// add a url tag to the files
    ///
    /// if the tag value is not a valid url then the operation will fail
    #[arg(
        long,
        conflicts_with_all(["tag", "tag_url", "tag_num"]),
        value_parser(parse_url_tag)
    )]
    pub add_url: Vec<Tag>,

    /// add a number tag to the files
    ///
    /// if the tag value is not a valid integer then the operation will fail
    #[arg(
        long,
        conflicts_with_all(["tag", "tag_url", "tag_num"]),
        value_parser(parse_num_tag)
    )]
    pub add_num: Vec<Tag>,

    /// remove a tag from the files
    ///
    /// this will remove a file from the existing list of tags for the
    /// specified files. if the tag is not found then nothing will happen
    /// update files with new comment
    #[arg(
        short = 'd',
        long,
        conflicts_with_all(["tag", "tag_url", "tag_num"])
    )]
    pub drop: Vec<String>,

    /// remote all tags from the files
    #[arg(
        long,
        conflicts_with_all(["tag", "tag_url", "tag_num", "add", "add_url", "add_num", "drop"])
    )]
    pub drop_all: bool
}

impl TagArgs {
    pub fn update(&self, mut tags: TagsMap) -> TagsMap {
        if self.drop_all {
            TagsMap::new()
        } else if !self.tag.is_empty() ||
            !self.tag_url.is_empty() ||
            !self.tag_num.is_empty() {
            let mut rtn = TagsMap::from_iter(self.tag.iter().cloned());
            rtn.extend(self.tag_url.iter().cloned());
            rtn.extend(self.tag_num.iter().cloned());
            rtn
        } else if !self.add.is_empty() ||
            !self.add_url.is_empty() ||
            !self.add_num.is_empty() ||
            !self.drop.is_empty() {
            for tag in &self.drop {
                tags.remove(tag);
            }

            tags.extend(self.add.iter().cloned());
            tags.extend(self.add_url.iter().cloned());
            tags.extend(self.add_num.iter().cloned());

            tags
        } else {
            tags
        }
    }
}

pub type TagsMap = HashMap<String, Option<TagValue>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagValue {
    Number(i64),
    Url(url::Url),
    Simple(String),
}

impl TagValue {
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
            TagValue::Number(v) => write!(f, "{}", v),
            TagValue::Url(v) => write!(f, "{}", v),
            TagValue::Simple(v) => write!(f, "{}", v),
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
            return Err(format!("tag name is empty"));
        }

        if value.is_empty() {
            Ok((name.into(), None))
        } else {
            Ok((name.into(), Some(value.into())))
        }
    } else {
        if arg.is_empty() {
            return Err(format!("tag is empty"));
        }

        Ok((arg.into(), None))
    }
}

pub fn parse_url_tag(arg: &str) -> Result<Tag, String> {
    if let Some((name, value)) = arg.split_once(':') {
        if name.is_empty() {
            return Err(format!("tag name is empty"));
        }

        if value.is_empty() {
            return Err(format!("missing url data"));
        }

        match TagValue::parse_url(value) {
            Ok(url) => Ok((name.into(), Some(url))),
            Err(err) => {
                Err(format!("invalid url provided: \"{}\" {}", value, err))
            }
        }
    } else {
        Err(format!("missing tag value"))
    }
}

pub fn parse_num_tag(arg: &str) -> Result<Tag, String> {
    if let Some((name, value)) = arg.split_once(':') {
        if name.is_empty() {
            return Err(format!("tag name is empty"));
        }

        if value.is_empty() {
            return Err(format!("missing num data"));
        }

        match TagValue::parse_num(value) {
            Ok(url) => Ok((name.into(), Some(url))),
            Err(err) => {
                Err(format!("invalid num provided: \"{}\" {}", value, err))
            }
        }
    } else {
        Err(format!("missing tag value"))
    }
}

