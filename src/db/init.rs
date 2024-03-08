use clap::Args;
use anyhow::Context;

use crate::fs;
use crate::db;
use crate::path;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// the type of db file to initalize
    #[arg(long, default_value = "json")]
    format: db::Format,
}

pub fn init_db(args: InitArgs) -> anyhow::Result<()> {
    let fsm_dir = path::get_cwd().join(".fsm");

    if let Some(fsm_metadata) = fs::get_metadata(&fsm_dir)
        .context("failed to retrieve metadata for .fsm directory")? {
        log::info!(".fsm entry already exists");

        if !fsm_metadata.is_dir() {
            return Err(anyhow::anyhow!(".fsm is not a directory"));
        }

        log::info!("checking for existing db");

        for format in db::FORMAT_LIST {
            let db_file = fsm_dir.join(format.file_name());

            let Some(metadata) = fs::get_metadata(&db_file)
                .context("io error when checking for db file")? else {
                continue;
            };

            if metadata.is_file() {
                println!("a db file already exists");
                return Ok(());
            } else if !metadata.is_file() {
                return Err(anyhow::anyhow!("a file system item exists with the name of a db file"));
            }
        }
    } else {
        log::info!("creating .fsm directory");

        std::fs::create_dir(&fsm_dir)
            .context("failed to create .fsm directory")?;
    }

    log::info!("creating db file");

    let db_file = fsm_dir.join(args.format.file_name());

    db::Context::create(db_file, args.format)
        .context("failed to save new db instance")?;

    Ok(())
}
