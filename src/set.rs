use clap::Args;

use crate::tags;
use crate::file;
use crate::db;

#[derive(Debug, Args)]
pub struct SetArgs {
    #[command(flatten)]
    tags: tags::TagArgs,

    /// sets a comment to the files
    #[arg(short = 'c', long, conflicts_with("drop_comment"))]
    comment: Option<String>,

    /// removes the comment from the files
    #[arg(long, conflicts_with("comment"))]
    drop_comment: bool,

    #[command(flatten)]
    file_list: file::FileList,
}

pub fn set_data(args: SetArgs) -> anyhow::Result<()> {
    let mut working_set = db::WorkingSet::new();

    for path_result in args.file_list.get_canon()? {
        let Some(path) = file::log_path_result(path_result) else {
            continue;
        };

        working_set.add_file(path)?;
    }

    for (file, db_path) in working_set.files {
        let db = working_set.dbs.get_mut(&db_path).unwrap();

        if let Some(existing) = db.inner.files.get_mut(&file) {
            log::info!("updating \"{}\" in db \"{}\"", file.display(), db_path.display());

            args.tags.update(&mut existing.tags);

            if args.drop_comment {
                existing.comment = None;
            } else if let Some(comment) = &args.comment {
                existing.comment = Some(comment.clone());
            }
        } else {
            log::info!("adding \"{}\" to db \"{}\"", file.display(), db_path.display());

            let mut data = db::FileData::default();

            args.tags.update(&mut data.tags);

            if args.drop_comment {
                data.comment = None;
            } else if let Some(comment) = &args.comment {
                data.comment = Some(comment.clone());
            }

            db.inner.files.insert(file, data);
        }
    }

    for (path, db) in working_set.dbs {
        log::debug!("db: {}\n{:#?}", path.display(), db);

        db.save()?;
    }

    Ok(())
}
