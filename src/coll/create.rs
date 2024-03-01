use std::collections::BTreeSet;

use clap::Args;

use crate::db;

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// the name of the new collection to create
    name: String
}

pub fn create_coll(args: CreateArgs) -> anyhow::Result<()> {
    let mut db_data = db::Db::cwd_load()?;

    if db_data.inner.collections.contains_key(&args.name) {
        println!("the specified collection already exists");
        return Ok(());
    }

    db_data.inner.collections.insert(args.name, BTreeSet::new());

    db_data.save()?;

    Ok(())
}
