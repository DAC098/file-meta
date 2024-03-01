use clap::Args;

use crate::db;
use crate::file;

#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// the name of the collection to update
    name: String,

    #[command(flatten)]
    file_list: file::FileList,
}

pub fn update_coll(args: UpdateArgs) -> anyhow::Result<()> {
    let mut db = db::Db::cwd_load()?;

    {
        let Some(mut coll) = db.inner.collections.remove(&args.name) else {
            println!("collection not found");
            return Ok(());
        };

        for path_result in args.file_list.get_canon()? {
            let Some(path) = file::log_path_result(path_result) else {
                continue;
            };

            let Some(adjusted) = db.maybe_common_root(&path) else {
                continue;
            };

            coll.insert(adjusted);
        }

        db.inner.collections.insert(args.name, coll);
    }

    db.save()?;

    Ok(())
}
