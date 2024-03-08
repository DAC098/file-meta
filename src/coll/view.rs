use clap::Args;

use crate::db;

#[derive(Debug, Args)]
pub struct ViewArgs {
    /// the name of a specific collection to view
    name: Option<String>,

    /// will display the files attached to a collection
    #[arg(short, long)]
    files: bool,
}

pub fn view_coll(args: ViewArgs) -> anyhow::Result<()> {
    let context = db::Context::cwd_load()?;

    if let Some(lookup) = args.name {
        let Some(files) = context.db.collections.get(&lookup) else {
            println!("collection not found");
            return Ok(());
        };

        println!("{}: {} files", lookup, files.len());

        if args.files {
            for file in files {
                println!("{}", file);
            }
        }
    } else {
        for (name, files) in &context.db.collections {
            println!("{}: {} files", name, files.len());

            if args.files {
                for file in files {
                    println!("{}", file);
                }
            }
        }
    }

    Ok(())
}
