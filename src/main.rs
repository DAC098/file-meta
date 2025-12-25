use clap::{Parser, Subcommand};

mod fs;
mod logging;
mod path;
mod time;

mod db;
mod tags;

mod coll;
mod delete;
mod get;
mod r#move;
mod open;
mod set;

/// a command line utility for managing additional data for files on the file
/// system
///
/// this provides additional methods of storing data for a file/directory
/// under a specified parent directory. store tags with an optional value that
/// can be opened later by the utility. create comments for additional context
/// if needed. specify collections of items you want grouped together that is
/// outside of the normal directory structure.
#[derive(Debug, Parser)]
#[command(max_term_width(80))]
struct AppArgs {
    #[command(subcommand)]
    cmd: Cmd,

    /// verbose logging for commands
    #[arg(short = 'V', long, conflicts_with("debug"))]
    verbose: bool,

    /// debug logging for commands
    #[arg(long, conflicts_with("verbose"))]
    debug: bool,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// retrieves information for the specified files
    Get(get::GetArgs),

    /// updates information for the specified files
    Set(set::SetArgs),

    /// moves a specified entry to another
    Move(r#move::MoveArgs),

    /// deletes entries from the database
    Delete(delete::DeleteArgs),

    /// attempts to open up the value of a tag or file from a collection
    Open(open::OpenArgs),

    /// manages collections in the db
    Coll(coll::CollectionArgs),

    /// manages db itself
    Db(db::DbArgs),
}

fn main() -> anyhow::Result<()> {
    path::set_cwd()?;
    env_logger::init();

    let args = AppArgs::parse();

    if args.verbose {
        log::set_max_level(log::LevelFilter::Info);
    } else if args.debug {
        log::set_max_level(log::LevelFilter::Debug);
    }

    match args.cmd {
        Cmd::Get(get_args) => get::get_data(get_args),
        Cmd::Set(set_args) => set::set_data(set_args),
        Cmd::Move(move_args) => r#move::move_data(move_args),
        Cmd::Delete(delete_args) => delete::delete_data(delete_args),
        Cmd::Open(open_args) => open::open(open_args),
        Cmd::Coll(coll_args) => coll::manage(coll_args),
        Cmd::Db(db_args) => db::manage(db_args),
    }
}
