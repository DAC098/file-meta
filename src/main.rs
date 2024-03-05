use clap::{Parser, Subcommand};

mod logging;
mod path;
mod fs;

mod tags;
mod db;

mod get;
mod set;
mod rename;
mod delete;
mod open;
mod coll;

#[derive(Debug, Parser)]
struct AppArgs {
    #[command(subcommand)]
    cmd: FileCmd,

    /// verbose logging for commands
    #[arg(short = 'V', long, conflicts_with("debug"))]
    verbose: bool,

    /// debug logging for commands
    #[arg(long, conflicts_with("verbose"))]
    debug: bool,
}

#[derive(Debug, Subcommand)]
enum FileCmd {
    /// retrieves information for the specified files
    Get(get::GetArgs),

    /// updates information for the specified files
    Set(set::SetArgs),

    /// renames a specified entry
    Rename(rename::RenameArgs),

    /// deletes entries from the database
    Delete(delete::DeleteArgs),

    /// attempts to open up the value of a tag
    Open(open::OpenArgs),

    /// manages collections in the db
    Coll(coll::CollectionArgs),

    /// manages db itself
    Db(db::DbArgs),
}

const RUST_LOG_ENV: &str = "RUST_LOG";

fn main() -> anyhow::Result<()> {
    path::set_cwd()?;

    let args = AppArgs::parse();

    if std::env::var_os(RUST_LOG_ENV).is_none() {
        if args.verbose {
            std::env::set_var(RUST_LOG_ENV, "info");
        } else if args.debug {
            std::env::set_var(RUST_LOG_ENV, "debug");
        }
    }

    env_logger::init();

    match args.cmd {
        FileCmd::Get(get_args) => get::get_data(get_args),
        FileCmd::Set(set_args) => set::set_data(set_args),
        FileCmd::Rename(rename_args) => rename::rename_data(rename_args),
        FileCmd::Delete(delete_args) => delete::delete_data(delete_args),
        FileCmd::Open(open_args) => open::open(open_args),
        FileCmd::Coll(coll_args) => coll::manage(coll_args),
        FileCmd::Db(db_args) => db::manage(db_args),
    }
}
