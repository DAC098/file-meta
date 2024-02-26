use clap::Args;
use anyhow::Context;

use crate::file;

#[derive(Debug, Args)]
pub struct OpenArgs {
    /// the desired tag to open
    tag: String,

    #[command(flatten)]
    file_list: file::FileList,
}

pub fn open_tag(args: OpenArgs) -> anyhow::Result<()> {
    for file_result in args.file_list.get_files()? {
        let file = match file_result {
            Ok(f) => f,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };

        if let Some(maybe_value) = file.data.tags.get(&args.tag) {
            let Some(value) = &maybe_value else {
                println!("{} {} has no value", file.ref_path().display(), args.tag);
                continue;
            };

            let url = match value {
                file::tags::TagValue::Url(url) => url.to_string(),
                _ => {
                    println!("{} {} is not a valid url", file.ref_path().display(), args.tag);
                    continue;
                }
            };

            if let Err(err) = opener::open(&url).context("failed to open url") {
                println!("{}", err);
            }
        } else {
            println!("{} {} does not exist", file.ref_path().display(), args.tag);
        }
    }

    Ok(())
}
