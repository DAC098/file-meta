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
    let mut db = db::Db::cwd_load()?;

    let (curr_path, curr_entry) = db.rel_to_db(args.current)?.into();
    let (rename_path, rename_entry) = db.rel_to_db(args.renamed)?.into();

    let Some(found) = db.inner.files.remove(&curr_entry) else {
        println!("current not found in db: {}", curr_path.display());
        return Ok(());
    };

    if args.exists && !fs::check_exists(&rename_path)? {
        println!("the renamed path does not exist: {}", rename_path.display());
        return Ok(());
    }

    if let Some(_exists) = db.inner.files.get_mut(&rename_entry) {
        println!("renamed already exists in db: {}", rename_entry);
    } else {
        db.inner.files.insert(rename_entry, found);
    }

    db.save()?;

    Ok(())
}
