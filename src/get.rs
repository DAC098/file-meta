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

    /// the file(s) to retrieve data for
    #[arg(trailing_var_arg = true, num_args(1..))]
    files: Vec<PathBuf>,
}

pub fn get_data(args: GetArgs) -> anyhow::Result<()> {
    let files_len = args.files.len();
    let db = db::Db::cwd_load()?;

    for path_result in db.relative_to_db(&args.files) {
        let Some(path) = logging::log_result(path_result) else {
            continue;
        };

        let Some(adjusted) = logging::log_result(path.to_db()) else {
            continue;
        };

        let Some(existing) = db.inner.files.get(adjusted) else {
            continue;
        };

        if files_len > 1 {
            println!("{}", adjusted.display());
        }

        if !args.no_tags {
            print_tags(&existing.tags);
        }

        if !args.no_comment {
            if let Some(comment) = &existing.comment {
                println!("comment: {}", comment);
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
