use clap::{Args, Subcommand};

mod view;
mod create;
mod delete;
mod update;

#[derive(Debug, Args)]
pub struct CollectionArgs {
    #[command(subcommand)]
    cmd: ManageCmd,
}

#[derive(Debug, Subcommand)]
enum ManageCmd {
    /// view information about a collection or a group of collections
    View(view::ViewArgs),
    /// create a new collection
    Create(create::CreateArgs),
    /// delete a given collection
    Delete(delete::DeleteArgs),
    /// update a given collection
    Update(update::UpdateArgs),
}

pub fn manage(args: CollectionArgs) -> anyhow::Result<()> {
    match args.cmd {
        ManageCmd::View(view_args) => view::view_coll(view_args),
        ManageCmd::Create(create_args) => create::create_coll(create_args),
        ManageCmd::Delete(delete_args) => delete::delete_coll(delete_args),
        ManageCmd::Update(update_args) => update::update_coll(update_args),
    }
}
