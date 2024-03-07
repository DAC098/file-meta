use std::path::{Path, PathBuf};

use clap::Args;
use anyhow::Context;

use crate::logging;
use crate::tags;
use crate::db;

#[derive(Debug, Args)]
pub struct OpenArgs {
    /// attempts to open up a tag in the db itself
    #[arg(long, requires("tag"))]
    self_: bool,

    /// name of the collection to open
    #[arg(short, long)]
    coll: Option<String>,

    /// the desired tag to open
    #[arg(short, long)]
    tag: Option<String>,

    /// the list of files to open
    ///
    /// if a collection has been specified then a list of files is not needed.
    #[arg(
        trailing_var_arg = true,
        required_unless_present_any(["coll", "self_"])
    )]
    files: Vec<PathBuf>,
}

pub fn open(args: OpenArgs) -> anyhow::Result<()> {
    let db = db::Db::cwd_load()?;

    if args.self_ {
        let tag = args.tag.as_ref().unwrap();

        if let Some(value) = retrieve_tag_value(db.root(), tag, &db.inner.tags) {
            open_tag(db.root(), tag, value);
        }
    }

    if let Some(name) = &args.coll {
        let Some(coll) = db.inner.collections.get(name) else {
            println!("collection not found");
            return Ok(());
        };

        for file in coll {
            if let Some(tag) = &args.tag {
                let Some(existing) = db.inner.files.get(file) else {
                    log::info!("file not found in db: {}", file.display());
                    continue;
                };

                if let Some(value) = retrieve_tag_value(file, tag, &existing.tags) {
                    open_tag(file, tag, value);
                }
            } else {
                let full_path = db.root().join(file);

                log::info!("opening file: {}", full_path.display());

                if let Err(err) = open::that_detached(&full_path).context("failed to open file") {
                    println!("{}", err);
                }
            }
        }
    } else if let Some(tag) = &args.tag {
        for path_result in db.rel_to_db_list(&args.files) {
            let Some(path) = logging::log_result(path_result) else {
                continue;
            };

            let Some(adjusted) = logging::log_result(path.to_db()) else {
                continue;
            };

            let Some(existing) = db.inner.files.get(adjusted) else {
                log::info!("{} {} does not exist", adjusted.display(), tag);
                continue;
            };

            if let Some(value) = retrieve_tag_value(adjusted, tag, &existing.tags) {
                open_tag(adjusted, tag, value);
            }
        }
    }

    Ok(())
}

fn retrieve_tag_value<'a>(file: &Path, tag: &str, map: &'a tags::TagsMap) -> Option<&'a tags::TagValue> {
    let Some(maybe) = map.get(tag) else {
        log::info!("{} {} does not exist", file.display(), tag);
        return None;
    };

    let Some(value) = maybe else {
        log::info!("{} {} has no value", file.display(), tag);
        return None;
    };

    Some(value)
}

fn open_tag(file: &Path, tag: &str, value: &tags::TagValue) {
    let url = match value {
        tags::TagValue::Url(url) => url.to_string(),
        _ => {
            log::info!("{} {} is not a valid url", file.display(), tag);
            return;
        }
    };

    log::info!("opening tag \"{}\" for file \"{}\"", tag, file.display());

    if let Err(err) = open::that_detached(&url).context("failed to open url") {
        println!("{}", err);
    }
}
