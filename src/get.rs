use std::collections::BinaryHeap;

use clap::Args;

use crate::tags;
use crate::db;
use crate::file;

#[derive(Debug, Args)]
pub struct GetArgs {
    /// does not output tags for files
    #[arg(long, conflicts_with("no_comment"))]
    no_tags: bool,

    /// does not output comments for files
    #[arg(long, conflicts_with("no_tags"))]
    no_comment: bool,

    #[command(flatten)]
    file_list: file::FileList,
}

pub fn get_data(args: GetArgs) -> anyhow::Result<()> {
    let mut working_set = db::WorkingSet::new();

    for path_result in args.file_list.get_canon()? {
        let Some(path) = file::log_path_result(path_result) else {
            continue;
        };

        working_set.add_file(path)?;
    }

    let files_len = working_set.files.len();

    for (file, db_path) in &working_set.files {
        let db = working_set.dbs.get(db_path).unwrap();

        let Some(existing) = db.inner.files.get(file) else {
            continue;
        };

        if files_len > 1 {
            println!("{}", file.display());
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
