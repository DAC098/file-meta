use anyhow::Context;
use clap::Args;

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
    let context = db::Context::cwd_load()?;

    if args.json {
        if args.pretty {
            serde_json::to_writer_pretty(std::io::stdout(), &context.db)
                .context("failed writing db to output")?;
        } else {
            serde_json::to_writer(std::io::stdout(), &context.db)
                .context("failed writing db to output")?;
        }
    } else {
        if args.pretty {
            println!("{:#?}", context.db);
        } else {
            println!("{:?}", context.db);
        }
    }

    Ok(())
}
