use std::path::PathBuf;

use clap::Args;

use crate::logging;
use crate::db;

#[derive(Debug, Args)]
pub struct PushArgs {
    /// the name of the collection to push files to
    name: String,

    /// the file(s) to push
    #[arg(trailing_var_arg(true), num_args(1..))]
    files: Vec<PathBuf>,
}

pub fn push_coll(args: PushArgs) -> anyhow::Result<()> {
    let mut db = db::Db::cwd_load()?;
    let files_iter = db.rel_to_db_list(&args.files);

    let Some(coll) = db.inner.collections.get_mut(&args.name) else {
        println!("collection not found");
        return Ok(());
    };

    for path_result in files_iter {
        let Some(path) = logging::log_result(path_result) else {
            continue;
        };

        let Some(adjusted) = logging::log_result(path.to_db()) else {
            continue;
        };

        coll.insert(adjusted.into());
    }

    db.save()?;

    Ok(())
}
