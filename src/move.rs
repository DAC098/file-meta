use std::path::PathBuf;

use anyhow::Context as _;
use clap::Args;

use crate::db::{self, MetaContainer as _};
use crate::fs;

#[derive(Debug, Args)]
pub struct MoveArgs {
    /// moves only tags
    #[arg(long, conflicts_with("comment"))]
    tags: bool,

    /// moves only the comment
    #[arg(long, conflicts_with("tags"))]
    comment: bool,

    /// moves data from the db to the destination
    #[arg(long, conflicts_with_all(["from", "to_self"]))]
    from_self: bool,

    /// the source file item
    #[arg(short, long, required_unless_present("from_self"))]
    from: Option<PathBuf>,

    /// checks to see if the destination exists
    #[arg(long)]
    exists: bool,

    /// moves data to the db from the source
    #[arg(long, conflicts_with_all(["to", "from_self"]))]
    to_self: bool,

    /// the destination file item
    #[arg(short, long,required_unless_present("to_self"))]
    to: Option<PathBuf>
}

fn get_src_entry(context: &mut db::Context, path: PathBuf) -> anyhow::Result<db::FileData> {
    let (src_path, src_entry) = context.rel_to_db(path)?.into();

    log::info!("moving from entry: {}", src_entry);

    context.db.files.remove(&src_entry)
        .with_context(|| format!("source not found in db: {}", src_path.display()))
}

fn get_dst_entry<'a>(context: &'a mut db::Context, path: PathBuf, check_exists: bool) -> anyhow::Result<&'a mut db::FileData> {
    let (dst_path, dst_entry) = context.rel_to_db(path)?.into();

    if check_exists && !fs::check_exists(&dst_path)? {
        return Err(anyhow::anyhow!("the destination path does not exist: {}", dst_path.display()));
    }

    log::info!("retrieving entry: {}", dst_entry);

    Ok(context.db.files.entry(dst_entry)
        .and_modify(db::FileData::update_ts)
        .or_default())
}

pub fn move_data(args: MoveArgs) -> anyhow::Result<()> {
    let mut context = db::Context::cwd_load()?;

    if args.tags {
        let src_tags = if let Some(from) = args.from {
            get_src_entry(&mut context, from)?.take_tags()
        } else {
            log::info!("moving tags from db");

            context.db.take_tags()
        };

        if let Some(to) = args.to {
            get_dst_entry(&mut context, to, args.exists)?
                .tags
                .extend(src_tags);
        } else {
            log::info!("updating db");

            context.db.update_ts();
            context.db.tags.extend(src_tags);
        }
    } else if args.comment {
        let src_comment = if let Some(from) = args.from {
            get_src_entry(&mut context, from)?.take_comment()
        } else {
            log::info!("moving comment from db");

            context.db.take_comment()
        };

        if let Some(to) = args.to {
            let found = get_dst_entry(&mut context, to, args.exists)?;

            if let Some(comment) = src_comment {
                found.comment = Some(comment);
            } else {
                log::info!("comment is empty");
            }
        } else {
            log::info!("updating db");

            if let Some(comment) = src_comment {
                context.db.comment = Some(comment);
            } else {
                log::info!("comment is empty");
            }
        }
    } else {
        let (src_tags, src_comment) = if let Some(from) = args.from {
            get_src_entry(&mut context, from)?.take_tags_comment()
        } else {
            log::info!("moving data from db");

            context.db.take_tags_comment()
        };

        if let Some(to) = args.to {
            let found = get_dst_entry(&mut context, to, args.exists)?;

            if let Some(comment) = src_comment {
                found.comment = Some(comment);
            }

            found.tags.extend(src_tags);
        } else {
            log::info!("updating db");

            context.db.update_ts();
            context.db.tags.extend(src_tags);

            if let Some(comment) = src_comment {
                context.db.comment = Some(comment);
            }
        }
    }

    context.save()?;

    Ok(())
}
