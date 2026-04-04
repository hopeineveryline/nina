pub mod check;
pub mod clone_cmd;
pub mod init;
pub mod lock;
pub mod show;
pub mod update;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct FlakeArgs {
    #[command(subcommand)]
    pub command: FlakeCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum FlakeCommand {
    Init(init::InitArgs),
    Update(update::UpdateArgs),
    Check(check::CheckArgs),
    Show(show::ShowArgs),
    Lock(lock::LockArgs),
    Clone(clone_cmd::CloneArgs),
}

pub async fn run(ctx: &AppContext, args: FlakeArgs) -> Result<()> {
    match args.command {
        FlakeCommand::Init(args) => init::run(ctx, args).await,
        FlakeCommand::Update(args) => update::run(ctx, args).await,
        FlakeCommand::Check(args) => check::run(ctx, args).await,
        FlakeCommand::Show(args) => show::run(ctx, args).await,
        FlakeCommand::Lock(args) => lock::run(ctx, args).await,
        FlakeCommand::Clone(args) => clone_cmd::run(ctx, args).await,
    }
}
