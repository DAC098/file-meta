use clap::Args;
use anyhow::Context;

use crate::file;

#[derive(Debug, Args)]
pub struct SetArgs {
    /// set a tag to the files
    ///
    /// this will override all previously set tags for the files to only
    /// include the provided tags
    #[arg(
        short,
        long,
        conflicts_with_all(["add_tag", "drop_tag"]),
        value_parser(file::parse_tag)
    )]
    tag: Vec<file::Tag>,

    /// add a tag to the files
    ///
    /// this will add to the existing list of tags for the specified files
    #[arg(
        short = 'a',
        long,
        conflicts_with("tag"),
        value_parser(file::parse_tag)
    )]
    add_tag: Vec<file::Tag>,

    /// remove a tag from the files
    ///
    /// this will remove a file from the existing list of tags for the
    /// specified files. if the tag is not found then nothing will happen
    /// update files with new comment
    #[arg(short = 'd', long, conflicts_with("tag"))]
    drop_tag: Vec<String>,

    /// sets a comment to the files
    #[arg(short = 'c', long)]
    comment: Option<String>,

    #[command(flatten)]
    file_list: file::FileList,
}

pub fn set_data(args: SetArgs) -> anyhow::Result<()> {
    let files = args.file_list.get_files()?;

    for mut file in files {
        if !args.tag.is_empty() {
            file.data.tags = file::TagsMap::from_iter(args.tag.iter().cloned());
        } else if !args.add_tag.is_empty() || !args.drop_tag.is_empty() {
            for tag in &args.drop_tag {
                file.data.tags.remove(tag);
            }

            file.data.tags.extend(args.add_tag.iter().cloned());
        }

        if let Some(comment) = &args.comment {
            file.data.comment = Some(comment.clone());
        }

        file.save()
            .with_context(|| format!("failed to save changes for file: \"{}\"", file.ref_path().display()))?;
    }

    Ok(())
}
