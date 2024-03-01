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
    tag: Option<String>,

    /// the list of files to open
    ///
    /// if a collection has been specified then a list of files is not needed.
    #[arg(
        trailing_var_arg = true,
        required_unless_present("coll")
    )]
    file_list: Vec<PathBuf>,
}

pub fn open_tag(args: OpenArgs) -> anyhow::Result<()> {
    let db = db::Db::cwd_load()?;

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

                let Some(maybe) = existing.tags.get(tag) else {
                    log::info!("{} {} does not exist", file.display(), tag);
                    continue;
                };

                let Some(value) = maybe else {
                    log::info!("{} {} does not exist", file.display(), tag);
                    continue;
                };

                let url = match value {
                    tags::TagValue::Url(url) => url.to_string(),
                    _ => {
                        log::info!("{} {} is not a valid url", file.display(), tag);
                        continue;
                    }
                };

                log::info!("opening tag \"{}\" for file \"{}\"", tag, file.display());

                if let Err(err) = opener::open(&url).context("failed to open url") {
                    println!("{}", err);
                }
            } else {
                let full_path = db.root().join(file);

                log::info!("opening file: {}", full_path.display());

                if let Err(err) = opener::open(&full_path).context("failed to open file") {
                    println!("{}", err);
                }
            }
        }
    } else if let Some(tag) = &args.tag {
        for path_result in file::CanonFiles::new(&args.file_list)? {
            let Some(path) = file::log_path_result(path_result) else {
                continue;
            };

            let Some(adjusted) = db.maybe_common_root(&path) else {
                continue;
            };

            let Some(existing) = db.inner.files.get(&adjusted) else {
                log::info!("{} {} does not exist", adjusted.display(), tag);
                continue;
            };

            let Some(maybe) = existing.tags.get(tag) else {
                log::info!("{} {} does not exist", adjusted.display(), tag);
                continue;
            };

            let Some(value) = maybe else {
                log::info!("{} {} has no value", adjusted.display(), tag);
                continue;
            };

            let url = match value {
                tags::TagValue::Url(url) => url.to_string(),
                _ => {
                    log::info!("{} {} is not a valid url", adjusted.display(), tag);
                    continue;
                }
            };

            if let Err(err) = opener::open(&url).context("failed to open url") {
                println!("{}", err);
            }
        }
    }

    Ok(())
}
