use std::collections::BTreeSet;

use clap::Args;

use crate::db;

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// the name of the new collection to create
    name: String,
}

pub fn create_coll(args: CreateArgs) -> anyhow::Result<()> {
    let mut context = db::Context::cwd_load()?;

    if context.db.collections.contains_key(&args.name) {
        println!("the specified collection already exists");
        return Ok(());
    }

    context.db.collections.insert(args.name, BTreeSet::new());

    context.save()?;

    Ok(())
}
