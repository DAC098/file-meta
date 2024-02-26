use clap::Args;
use anyhow::Context;

use crate::file;

#[derive(Debug, Args)]
pub struct SetArgs {
    #[command(flatten)]
    tags: file::tags::TagArgs,

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
    for file_result in args.file_list.get_files()? {
        let mut file = match file_result {
            Ok(f) => f,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };

        file.data.tags = args.tags.update(file.data.tags);

        if args.drop_comment {
            file.data.comment = None;
        } else if let Some(comment) = &args.comment {
            file.data.comment = Some(comment.clone());
        }

        file.save()
            .with_context(|| format!("failed to save changes for file: \"{}\"", file.ref_path().display()))?;
    }

    Ok(())
}
