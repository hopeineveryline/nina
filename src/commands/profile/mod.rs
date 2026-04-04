pub mod install;
pub mod list;
pub mod remove;
pub mod upgrade;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ProfileCommand {
    List(list::ListArgs),
    Install(install::InstallArgs),
    Remove(remove::RemoveArgs),
    Upgrade(upgrade::UpgradeArgs),
}

pub async fn run(ctx: &AppContext, args: ProfileArgs) -> Result<()> {
    match args.command {
        ProfileCommand::List(args) => list::run(ctx, args).await,
        ProfileCommand::Install(args) => install::run(ctx, args).await,
        ProfileCommand::Remove(args) => remove::run(ctx, args).await,
        ProfileCommand::Upgrade(args) => upgrade::run(ctx, args).await,
    }
}
