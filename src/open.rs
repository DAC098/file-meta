use std::path::PathBuf;

use clap::Args;
use anyhow::Context;

use crate::tags;
use crate::file;
use crate::db;

#[derive(Debug, Args)]
pub struct OpenArgs {
    /// name of the collection to open
    #[arg(long)]
    coll: Option<String>,

    /// the desired tag to open
    #[arg(long)]
    tag: String,

    /// the list of files to open
    #[arg(
        trailing_var_arg = true,
        required_unless_present("coll")
    )]
    file_list: Vec<PathBuf>,
}

pub fn open_tag(args: OpenArgs) -> anyhow::Result<()> {
    let db = db::Db::cwd_load()?;

    for path_result in file::CanonFiles::new(&args.file_list)? {
        let Some(path) = file::log_path_result(path_result) else {
            continue;
        };

        let Some(adjusted) = db.maybe_common_root(&path) else {
            continue;
        };

        let Some(existing) = db.inner.files.get(&adjusted) else {
            println!("{} {} does not exist", adjusted.display(), args.tag);
            continue;
        };

        let Some(maybe) = existing.tags.get(&args.tag) else {
            println!("{} {} does not exist", adjusted.display(), args.tag);
            continue;
        };

        let Some(value) = maybe else {
            println!("{} {} has no value", adjusted.display(), args.tag);
            continue;
        };

        let url = match value {
            tags::TagValue::Url(url) => url.to_string(),
            _ => {
                println!("{} {} is not a valid url", adjusted.display(), args.tag);
                continue;
            }
        };

        if let Err(err) = opener::open(&url).context("failed to open url") {
            println!("{}", err);
        }
    }

    Ok(())
}
