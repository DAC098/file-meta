use std::path::PathBuf;

use clap::Args;

use crate::db;
use crate::fs;

#[derive(Debug, Args)]
pub struct RenameArgs {
    /// checks to make sure that the renamed value exists
    #[arg(long)]
    exists: bool,

    /// current name of the entry
    current: PathBuf,

    /// the new name of the entry
    renamed: PathBuf,
}

pub fn rename_data(args: RenameArgs) -> anyhow::Result<()> {
    let mut context = db::Context::cwd_load()?;

    let (curr_path, curr_entry) = context.rel_to_db(args.current)?.into();
    let (rename_path, rename_entry) = context.rel_to_db(args.renamed)?.into();

    let Some(found) = context.db.files.remove(&curr_entry) else {
        println!("current not found in db: {}", curr_path.display());
        return Ok(());
    };

    if args.exists && !fs::check_exists(&rename_path)? {
        println!("the renamed path does not exist: {}", rename_path.display());
        return Ok(());
    }

    if let Some(_exists) = context.db.files.get_mut(&rename_entry) {
        println!("renamed already exists in db: {}", rename_entry);
    } else {
        context.db.files.insert(rename_entry, found);
    }

    context.save()?;

    Ok(())
}
