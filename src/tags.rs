use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use serde::{Serialize, Deserialize};
use url::Url;

pub type TagsMap = BTreeMap<String, Option<TagValue>>;

#[derive(Debug, thiserror::Error)]
#[error("the provided tag key contains invalid characters")]
pub struct InvalidTagChars;

pub const INVALID_CHARS: [char; 4] = ['\\', ':', ',', '!'];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TagKey(String);

impl TagKey {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl Display for TagKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl FromStr for TagKey {
    type Err = InvalidTagChars;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        for ch in value.chars() {
            if ch.is_control() ||
                ch.is_whitespace() ||
                INVALID_CHARS.contains(&ch)
            {
                return Err(InvalidTagChars);
            }
        }

        Ok(TagKey(value.to_owned()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagValue {
    Number(i64),
    Bool(bool),
    Url(url::Url),
    Simple(String),
}

impl TagValue {
    fn parse_num(value: &str) -> Result<Self, std::num::ParseIntError> {
        Ok(TagValue::Number(value.parse()?))
    }

    fn parse_bool(value: &str) -> Result<Self, std::str::ParseBoolError> {
        Ok(TagValue::Bool(value.parse()?))
    }

    fn parse_url(value: &str) -> Result<Self, url::ParseError> {
        Ok(TagValue::Url(Url::parse(value)?))
    }
}

impl Display for TagValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TagValue::Number(v) => write!(f, "{}", v),
            TagValue::Bool(v) => write!(f, "{}", v),
            TagValue::Url(v) => write!(f, "{}", v),
            TagValue::Simple(v) => write!(f, "{}", v),
        }
    }
}

impl From<&str> for TagValue {
    fn from(value: &str) -> Self {
        if let Ok(i64_value) = value.parse() {
            TagValue::Number(i64_value)
        } else if let Ok(bool_) = value.parse() {
            TagValue::Bool(bool_)
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

fn get_name_value<'a>(arg: &'a str) -> Result<(&'a str, &'a str), String> {
    if let Some((name, value)) = arg.split_once(':') {
        if name.is_empty() {
            return Err(format!("tag name is empty"));
        }

        if value.is_empty() {
            return Err(format!("missing url data"));
        }

        Ok((name, value))
    } else {
        Err(format!("missing tag value"))
    }
}

pub fn parse_url_tag(arg: &str) -> Result<Tag, String> {
    let (name, value) = get_name_value(arg)?;

    match TagValue::parse_url(value) {
        Ok(url) => Ok((name.into(), Some(url))),
        Err(err) => Err(format!("invalid url provided: {}", err))
    }
}

pub fn parse_num_tag(arg: &str) -> Result<Tag, String> {
    let (name, value) = get_name_value(arg)?;

    match TagValue::parse_num(value) {
        Ok(url) => Ok((name.into(), Some(url))),
        Err(err) => Err(format!("invalid num provided: {}", err))
    }
}

pub fn parse_bool_tag(arg: &str) -> Result<Tag, String> {
    let (name, value) = get_name_value(arg)?;

    match TagValue::parse_bool(value) {
        Ok(b) => Ok((name.into(), Some(b))),
        Err(err) => Err(format!("invalid bool provided: {}", err))
    }
}
