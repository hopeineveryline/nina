pub mod current;
pub mod delete;
pub mod list;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct GenArgs {
    #[command(subcommand)]
    pub command: GenCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum GenCommand {
    List(list::ListArgs),
    Current(current::CurrentArgs),
    Delete(delete::DeleteArgs),
}

pub async fn run(ctx: &AppContext, args: GenArgs) -> Result<()> {
    match args.command {
        GenCommand::List(args) => list::run(ctx, args).await,
        GenCommand::Current(args) => current::run(ctx, args).await,
        GenCommand::Delete(args) => delete::run(ctx, args).await,
    }
}
