use clap::Args;
use anyhow::Context;

use crate::tags;
use crate::file;
use crate::db;

#[derive(Debug, Args)]
pub struct OpenArgs {
    /// the desired tag to open
    tag: String,

    #[command(flatten)]
    file_list: file::FileList,
}

pub fn open_tag(args: OpenArgs) -> anyhow::Result<()> {
    let mut working_set = db::WorkingSet::new();

    for path_result in args.file_list.get_canon()? {
        let Some(path) = file::log_path_result(path_result) else {
            continue;
        };

        working_set.add_file(path)?;
    }

    for (file, db_path) in working_set.files {
        let db = working_set.dbs.get(&db_path).unwrap();

        let Some(existing) = db.inner.files.get(&file) else {
            println!("{} {} does not exist", file.display(), args.tag);
            continue;
        };

        let Some(maybe) = existing.tags.get(&args.tag) else {
            println!("{} {} does not exist", file.display(), args.tag);
            continue;
        };

        let Some(value) = maybe else {
            println!("{} {} has no value", file.display(), args.tag);
            continue;
        };

        let url = match value {
            tags::TagValue::Url(url) => url.to_string(),
            _ => {
                println!("{} {} is not a valid url", file.display(), args.tag);
                continue;
            }
        };

        if let Err(err) = opener::open(&url).context("failed to open url") {
            println!("{}", err);
        }
    }

    Ok(())
}
