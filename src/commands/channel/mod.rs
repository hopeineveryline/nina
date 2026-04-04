pub mod add;
pub mod list;
pub mod remove;
pub mod update;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct ChannelArgs {
    #[command(subcommand)]
    pub command: ChannelCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ChannelCommand {
    List(list::ListArgs),
    Add(add::AddArgs),
    Remove(remove::RemoveArgs),
    Update(update::UpdateArgs),
}

pub async fn run(ctx: &AppContext, args: ChannelArgs) -> Result<()> {
    match args.command {
        ChannelCommand::List(args) => list::run(ctx, args).await,
        ChannelCommand::Add(args) => add::run(ctx, args).await,
        ChannelCommand::Remove(args) => remove::run(ctx, args).await,
        ChannelCommand::Update(args) => update::run(ctx, args).await,
    }
}
