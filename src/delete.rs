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
            let full_path = root.join(&file);

            if let Some(m) = fs::get_metadata(&full_path)? {
                log::info!("file {} exists", file.display());

                updated.insert(file, data);
            } else {
                log::info!("removing {}", file.display());
            }
        }

        db.inner.files = updated;
    }

    for path_result in db.relative_to_db(&args.files) {
        let Some(path) = logging::log_result(path_result) else {
            continue;
        };

        let Some(adjusted) = logging::log_result(path.to_db()) else {
            continue;
        };

        log::info!("looking for: {}", adjusted.display());

        if let Some(removed) = db.inner.files.remove(adjusted) {
            log::info!("file not found in db: {}", path.display());
        } else {
            log::info!("file removed from db: {}", path.display());
        }
    }

    db.save()?;

    Ok(())
}
