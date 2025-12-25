use anyhow::Context;
use clap::Args;

use crate::db;

#[derive(Debug, Args)]
pub struct DropArgs {}

pub fn drop_db(_args: DropArgs) -> anyhow::Result<()> {
    let context = db::Context::cwd_load()?;

    log::info!("dropping db file: {}", context.path().display());

    std::fs::remove_file(context.path()).context("failed to remove db file")?;

    let dir = context.path().parent().unwrap();

    log::info!("dropping fsm directory: {}", dir.display());

    std::fs::remove_dir(dir).context("failed to remove .fsm directory")?;

    Ok(())
}
