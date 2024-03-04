use clap::{Parser, Subcommand};

mod fs;

mod file;
mod tags;
mod db;

mod get;
mod set;
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

    /// attempts to open up the value of a tag
    Open(open::OpenArgs),

    /// manages collections in the db
    Coll(coll::CollectionArgs),

    /// manages db itself
    Db(db::DbArgs),
}

const RUST_LOG_ENV: &str = "RUST_LOG";

fn main() -> anyhow::Result<()> {
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
        FileCmd::Open(open_args) => open::open_tag(open_args),
        FileCmd::Coll(coll_args) => coll::manage(coll_args),
        FileCmd::Db(db_args) => db::manage(db_args),
    }
}
