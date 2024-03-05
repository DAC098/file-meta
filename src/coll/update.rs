use std::path::PathBuf;
use clap::Args;

use crate::logging;
use crate::db;

#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// the name of the collection to update
    name: String,

    /// the file(s) to add to the collection
    #[arg(trailing_var_arg = true, num_args(1..))]
    files: Vec<PathBuf>,
}

pub fn update_coll(args: UpdateArgs) -> anyhow::Result<()> {
    let mut db = db::Db::cwd_load()?;

    {
        let path_iter = db.rel_to_db_list(&args.files);

        let Some(coll) = db.inner.collections.get_mut(&args.name) else {
            println!("collection not found");
            return Ok(());
        };

        for path_result in path_iter {
            let Some(path) = logging::log_result(path_result) else {
                continue;
            };

            let Some(adjusted) = logging::log_result(path.to_db()) else {
                continue;
            };

            coll.insert(adjusted.into());
        }
    }

    db.save()?;

    Ok(())
}
