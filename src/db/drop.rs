use clap::Args;
use anyhow::Context;

use crate::db;

#[derive(Debug, Args)]
pub struct DropArgs {}

pub fn drop_db(_args: DropArgs) -> anyhow::Result<()> {
    let db = db::Db::cwd_load()?;

    log::info!("dropping db file: {}", db.path().display());

    std::fs::remove_file(db.path())
        .context("failed to remove db file")?;

    let dir = db.path()
        .parent()
        .unwrap();

    log::info!("dropping fsm directory: {}", dir.display());

    std::fs::remove_dir(dir)
        .context("failed to remove .fsm directory")?;

    Ok(())
}
