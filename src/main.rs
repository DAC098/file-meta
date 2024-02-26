use std::path::PathBuf;

use clap::{Parser, Args, Subcommand};
use anyhow::Context;

mod file;
mod get;
mod set;
mod open;

#[derive(Debug, Parser)]
struct AppArgs {
    #[command(subcommand)]
    cmd: FileCmd,
}

#[derive(Debug, Subcommand)]
enum FileCmd {
    /// retrieves information for the specified files
    Get(get::GetArgs),

    /// updates information for the specified files
    Set(set::SetArgs),

    /// attempts to open up the value of a tag
    Open(open::OpenArgs),
}

fn main() -> anyhow::Result<()> {
    let args = AppArgs::parse();

    match args.cmd {
        FileCmd::Get(get_args) => get::get_data(get_args),
        FileCmd::Set(set_args) => set::set_data(set_args),
        FileCmd::Open(open_args) => open::open_tag(open_args),
    }
}
