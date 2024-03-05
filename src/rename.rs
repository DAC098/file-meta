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
    let current = db.rel_to_db(args.current)?;
    let current_adjusted = current.to_db()?;

    let rename = db.rel_to_db(args.renamed)?;
    let rename_adjusted = rename.to_db()?;

    let Some(found) = db.inner.files.remove(current_adjusted) else {
        println!("current not found in db: {}", current_adjusted.display());
        return Ok(());
    };

    if args.exists && !fs::check_exists(rename.full_path())? {
        println!("the renamed path does not exist: {}", rename.display());
        return Ok(());
    }

    if let Some(_exists) = db.inner.files.get_mut(rename_adjusted) {
        println!("renamed already exists in db: {}", rename_adjusted.display());
    } else {
        db.inner.files.insert(rename_adjusted.into(), found);
    }

    db.save()?;

    Ok(())
}
