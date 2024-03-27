use std::collections::BinaryHeap;
use std::path::PathBuf;
use std::fmt::Display;

use clap::Args;

use crate::logging;
use crate::tags;
use crate::db::{self, MetaContainer};

#[derive(Debug, Args)]
pub struct GetArgs {
    /// does not output tags for files
    #[arg(long, conflicts_with("no_comment"))]
    no_tags: bool,

    /// does not output comments for files
    #[arg(long, conflicts_with("no_tags"))]
    no_comment: bool,

    /// retrieves all known data in the db
    #[arg(long)]
    all: bool,

    /// retrieves data from the db itself
    #[arg(long = "self")]
    self_: bool,

    /// the file(s) to retrieve data for
    #[arg(
        trailing_var_arg(true),
        required_unless_present_any(["self_", "all"])
    )]
    files: Vec<PathBuf>,
}

pub fn get_data(args: GetArgs) -> anyhow::Result<()> {
    let context = db::Context::cwd_load()?;

    let print_title = if args.all {
        context.db.files.len() + 1
    } else if args.self_ {
        args.files.len() + 1
    } else {
        args.files.len()
    } > 1;

    if args.self_ || args.all {
        print_data(&context.root().display(), &context.db, &args, print_title);
    }

    if args.all {
        for (key, file) in &context.db.files {
            print_data(&key, file, &args, print_title);
        }
    } else {
        for path_result in context.rel_to_db_list(&args.files) {
            let Some(rel_path) = logging::log_result(path_result) else {
                continue;
            };

            let (_path, db_entry) = rel_path.into();

            let Some(existing) = context.db.files.get(&db_entry) else {
                println!("{db_entry} not found");
                continue;
            };

            print_data(&db_entry, existing, &args, print_title);
        }
    }

    Ok(())
}

#[inline]
fn print_entry<E>(entry: &E)
where
    E: Display + ?Sized
{
    println!("@ {entry}");
}

fn print_data<E, M>(entry: &E, container: &M, args: &GetArgs, print_title: bool)
where
    M: MetaContainer,
    E: Display + ?Sized,
{
    let mut printed_key = false;
    let mut print_ts = false;

    if !args.no_tags {
        if print_title {
            print_entry(entry);
            printed_key = true;
        }

        print_tags(container.tags());
        print_ts = true;
    }

    if !args.no_comment {
        if let Some(comment) = container.comment() {
            if print_title && !printed_key {
                print_entry(entry);
            }

            println!("comment: {comment}");
            print_ts = true;
        }
    }

    if print_ts {
        let local_offset = chrono::Local;

        if let Some(updated) = container.updated() {
            let local_updated = updated.with_timezone(&local_offset);

            println!("{local_updated}");
        } else {
            let local_created = container.created()
                .with_timezone(&local_offset);

            println!("{local_created}");
        }
    }
}

fn print_tags(tags: &tags::TagsMap) {
    let mut max_len = 0usize;
    let mut no_value = BinaryHeap::new();
    let mut with_value = BinaryHeap::new();

    for (key, value) in tags {
        if value.is_some() {
            with_value.push(key.clone());

            let chars_count = key.chars().count();

            if chars_count > max_len {
                max_len = key.chars().count();
            }
        } else {
            no_value.push(key.clone());
        }
    }

    for key in no_value.into_sorted_vec() {
        println!("{key}");
    }

    for key in with_value.into_sorted_vec() {
        let value = tags.get(&key)
            .unwrap()
            .as_ref()
            .unwrap();

        println!("{key:>max_len$}: {value}");
    }
}
