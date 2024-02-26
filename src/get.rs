use std::collections::BinaryHeap;

use clap::Args;

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
    let files = args.file_list.get_files()?;
    let files_len = files.len();

    for file in files {
        if files_len > 1 {
            println!("{}", file.ref_path().display());
        }

        if !args.no_tags {
            print_tags(&file.data.tags);
        }

        if !args.no_comment {
            if let Some(comment) = &file.data.comment {
                println!("comment: {}", comment);
            }
        }
    }

    Ok(())
}

fn print_tags(tags: &file::TagsMap) {
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
