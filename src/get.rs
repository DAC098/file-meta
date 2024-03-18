use std::collections::BinaryHeap;
use std::path::PathBuf;

use clap::Args;

use crate::logging;
use crate::tags;
use crate::db;

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

    let files_len = if args.all {
        context.db.files.len() + 1
    } else if args.self_ {
        args.files.len() + 1
    } else {
        args.files.len()
    };

    if args.self_ || args.all {
        let mut printed_key = false;

        if !args.no_tags {
            if files_len > 1 {
                println!("-- {}", context.root().display());
                printed_key = true;
            }

            print_tags(&context.db.tags);
        }

        if !args.no_comment {
            if let Some(comment) = &context.db.comment {
                if files_len > 1 && !printed_key {
                    println!("-- {}", context.root().display());
                }

                println!("comment: {}", comment);
            }
        }
    }

    if args.all {
        for (key, file) in &context.db.files {
            let mut printed_key = false;

            if !args.no_tags {
                if files_len > 1 {
                    println!("-- {}", key);
                    printed_key = true;
                }

                print_tags(&file.tags);
            }

            if !args.no_comment {
                if let Some(comment) = &file.comment {
                    if files_len > 1 && !printed_key {
                        println!("-- {}", key);
                    }

                    println!("comment: {}", comment);
                }
            }
        }
    } else {
        for path_result in context.rel_to_db_list(&args.files) {
            let Some(rel_path) = logging::log_result(path_result) else {
                continue;
            };

            let (_path, db_entry) = rel_path.into();

            let Some(existing) = context.db.files.get(&db_entry) else {
                continue;
            };

            let mut printed_key = false;

            if !args.no_tags {
                if files_len > 1 {
                    println!("-- {}", db_entry);
                    printed_key = true;
                }

                print_tags(&existing.tags);
            }

            if !args.no_comment {
                if let Some(comment) = &existing.comment {
                    if files_len > 1 && !printed_key {
                        println!("-- {}", db_entry);
                    }

                    println!("comment: {}", comment);
                }
            }
        }
    }

    Ok(())
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
        println!("{}", key);
    }

    for key in with_value.into_sorted_vec() {
        let value = tags.get(&key)
            .unwrap()
            .as_ref()
            .unwrap();

        println!("{:>max_len$}: {}", key, value);
    }
}
