use std::collections::BTreeSet;
use std::path::PathBuf;

use clap::Args;

use crate::logging;
use crate::db;
use crate::fs;

#[derive(Debug, Args)]
pub struct PopArgs {
    /// the name of the collection to pop files from
    name: String,

    /// drop files that do not exist
    #[arg(long)]
    no_exists:bool,

    /// the file(s) to pop
    #[arg(
        trailing_var_arg(true),
        required_unless_present("no_exists")
    )]
    files: Vec<PathBuf>,
}

pub fn pop_coll(args: PopArgs) -> anyhow::Result<()> {
    let mut db = db::Db::cwd_load()?;
    let root = db.root_copy();
    let files_iter = db.rel_to_db_list(&args.files);

    let Some(coll) = db.inner.collections.get_mut(&args.name) else {
        println!("collection not found");
        return Ok(());
    };

    if args.no_exists {
        let mut updated = BTreeSet::new();

        for file in coll.iter() {
            let full_path = root.join(&**file);

            if fs::check_exists(&full_path)? {
                log::info!("file {} exists", file);

                updated.insert(file.clone());
            } else {
                log::info!("removing {}", file);
            }
        }

        *coll = updated;
    }

    for path_result in files_iter {
        let Some(rel_path) = logging::log_result(path_result) else {
            continue;
        };

        let (_path, db_entry) = rel_path.into();

        coll.remove(&db_entry);
    }

    db.save()?;

    Ok(())
}
