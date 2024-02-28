use clap::{Parser, Subcommand};

mod fs;

mod file;
mod tags;
mod db;

mod get;
mod set;
mod open;
mod init;

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

    /// initializes a directory with an fsm db
    Init(init::InitArgs),
}

fn main() -> anyhow::Result<()> {
    let args = AppArgs::parse();

    if args.verbose {
        if std::env::var_os("RUST_LOG").is_none() {
            std::env::set_var("RUST_LOG", "info");
        }
    } else if args.debug {
        if std::env::var_os("RUST_LOG").is_none() {
            std::env::set_var("RUST_LOG", "debug");
        }
    }

    env_logger::init();

    match args.cmd {
        FileCmd::Get(get_args) => get::get_data(get_args),
        FileCmd::Set(set_args) => set::set_data(set_args),
        FileCmd::Open(open_args) => open::open_tag(open_args),
        FileCmd::Init(init_args) => init::init_db(init_args),
    }
}
