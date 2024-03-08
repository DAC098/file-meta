use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::Args;

use crate::logging;
use crate::fs;
use crate::db;

#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// will remove all entries that are missing the corresponding file
    #[arg(long)]
    not_exists: bool,

    /// the file(s) to remove from the database
    #[arg(
        trailing_var_arg = true,
        required_unless_present("not_exists")
    )]
    files: Vec<PathBuf>,
}

pub fn delete_data(args: DeleteArgs) -> anyhow::Result<()> {
    let mut db = db::Db::cwd_load()?;
    let root = db.root_copy();

    if args.not_exists {
        let mut updated = BTreeMap::new();

        for (file, data) in db.inner.files {
            let full_path = root.join(&*file);

            if fs::check_exists(&full_path)? {
                log::info!("file {} exists", file);

                updated.insert(file, data);
            } else {
                log::info!("removing {}", file);
            }
        }

        db.inner.files = updated;
    }

    for path_result in db.rel_to_db_list(&args.files) {
        let Some(rel_path) = logging::log_result(path_result) else {
            continue;
        };

        let (_path, db_entry) = rel_path.into();

        log::info!("looking for: {}", db_entry);

        if let Some(_removed) = db.inner.files.remove(&db_entry) {
            log::info!("file not found in db: {}", db_entry);
        } else {
            log::info!("file removed from db: {}", db_entry);
        }
    }

    db.save()?;

    Ok(())
}
