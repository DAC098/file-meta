use clap::Args;
use anyhow::Context;

use crate::db;

#[derive(Debug, Args)]
pub struct DumpArgs {
    /// dumps the database as json
    #[arg(long)]
    json: bool,

    /// pretty prints the output
    #[arg(long)]
    pretty: bool,
}

pub fn dump_db(args: DumpArgs) -> anyhow::Result<()> {
    let db_data = db::Db::cwd_load()?;

    if args.json {
        if args.pretty {
            serde_json::to_writer_pretty(std::io::stdout(), &db_data.inner)
                .context("failed writing db to output")?;
        } else {
            serde_json::to_writer(std::io::stdout(), &db_data.inner)
                .context("failed writing db to output")?;
        }
    } else {
        if args.pretty {
            println!("{:#?}", db_data.inner);
        } else {
            println!("{:?}", db_data.inner);
        }
    }

    Ok(())
}
