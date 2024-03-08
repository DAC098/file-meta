use clap::Args;

use crate::db;

#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// the name of the collection to delete
    name: String,

    /// display the list of files contained in the collection
    #[arg(short, long)]
    files: bool,
}

pub fn delete_coll(args: DeleteArgs) -> anyhow::Result<()> {
    let mut db_data = db::Db::cwd_load()?;

    let Some(files) = db_data.inner.collections.remove(&args.name) else {
        println!("collection not found");
        return Ok(());
    };

    db_data.save()?;

    if args.files {
        println!("{} files", files.len());

        for file in files {
            println!("{}", file);
        }
    }

    Ok(())
}
