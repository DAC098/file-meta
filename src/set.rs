use std::path::PathBuf;

use clap::Args;

use crate::logging;
use crate::tags;
use crate::db;

#[derive(Debug, Args)]
pub struct SetArgs {
    /// replaces all current tags with new ones
    ///
    /// will remove all currently set tags for the specified and replace
    /// them will the onces currently specified
    #[arg(long)]
    replace: bool,

    /// set a tag to the files
    ///
    /// this will override all previously set tags for the files to only
    /// include the provided tags
    #[arg(
        short,
        conflicts_with_all(["drop_all"]),
        value_parser(tags::parse_tag)
    )]
    tag: Vec<tags::Tag>,

    /// set a url tag to the files
    ///
    /// similar to a regular tag but if the tag value is not a valid url then
    /// the operation will fail
    #[arg(
        short = 'u',
        conflicts_with_all(["drop_all"]),
        value_parser(tags::parse_url_tag)
    )]
    tag_url: Vec<tags::Tag>,

    /// set a number tag to the files
    ///
    /// similar to the regular tag but if the tag value is not a valid integer
    /// then the operation will fail
    #[arg(
        short = 'n',
        conflicts_with_all(["drop_all"]),
        value_parser(tags::parse_num_tag)
    )]
    tag_num: Vec<tags::Tag>,

    /// set a bool tag to the files
    ///
    /// similar to the regular tag but if the tag value is not a valid bool
    /// then the operation will fail
    #[arg(
        short = 'b',
        conflicts_with_all(["drop_all"]),
        value_parser(tags::parse_bool_tag)
    )]
    tag_bool: Vec<tags::Tag>,

    /// remove a tag from the files
    ///
    /// this will remove a tag from the existing list of tags for the
    /// specified files. if the tag is not found then nothing will happen
    #[arg(short = 'd', long, conflicts_with_all(["drop_all"]))]
    drop: Vec<String>,

    /// remote all tags from the files
    #[arg(
        long,
        conflicts_with_all(["tag", "tag_url", "tag_num", "tag_bool", "drop"])
    )]
    drop_all: bool,

    /// sets a comment to the files
    #[arg(short = 'c', long, conflicts_with("drop_comment"))]
    comment: Option<String>,

    /// removes the comment from the files
    #[arg(long, conflicts_with("comment"))]
    drop_comment: bool,

    /// sets tags to the db itself
    #[arg(long = "self")]
    self_: bool,

    /// the file(s) to update data for
    #[arg(
        trailing_var_arg(true),
        required_unless_present("self_")
    )]
    files: Vec<PathBuf>,
}

fn has_tags(args: &SetArgs) -> bool {
    !args.tag.is_empty() ||
        !args.tag_url.is_empty() ||
        !args.tag_num.is_empty() ||
        !args.tag_bool.is_empty()
}

fn update_tags(args: &SetArgs, tags: &mut tags::TagsMap) {
    if args.drop_all {
        tags.clear();
    } else if has_tags(args) || !args.drop.is_empty() {
        if args.replace {
            tags.clear();
        } else {
            for tag in &args.drop {
                tags.remove(tag);
            }
        }

        tags.extend(args.tag.iter().cloned());
        tags.extend(args.tag_url.iter().cloned());
        tags.extend(args.tag_num.iter().cloned());
        tags.extend(args.tag_bool.iter().cloned());
    }
}

pub fn set_data(args: SetArgs) -> anyhow::Result<()> {
    let mut context = db::Context::cwd_load()?;

    if args.self_ {
        update_tags(&args, &mut context.db.tags);

        if args.drop_comment {
            context.db.comment = None;
        } else if let Some(comment) = &args.comment {
            context.db.comment = Some(comment.clone());
        }
    }

    for path_result in context.rel_to_db_list(&args.files) {
        let Some(rel_path) = logging::log_result(path_result) else {
            continue;
        };

        let (_path, db_entry) = rel_path.into();

        if let Some(existing) = context.db.files.get_mut(&db_entry) {
            log::info!("updating \"{}\"", db_entry);

            update_tags(&args, &mut existing.tags);

            if args.drop_comment {
                existing.comment = None;
            } else if let Some(comment) = &args.comment {
                existing.comment = Some(comment.clone());
            }

            existing.updated = Some(chrono::Utc::now());
        } else {
            log::info!("adding \"{}\"", db_entry);

            let mut data = db::FileData::default();

            update_tags(&args, &mut data.tags);

            if args.drop_comment {
                data.comment = None;
            } else if let Some(comment) = &args.comment {
                data.comment = Some(comment.clone());
            }

            context.db.files.insert(db_entry, data);
        }
    }

    context.save()?;

    Ok(())
}
