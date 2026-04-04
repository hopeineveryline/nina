pub mod disable;
pub mod enable;
pub mod list;
pub mod logs;
pub mod restart;
pub mod start;
pub mod status;
pub mod stop;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct ServiceArgs {
    #[command(subcommand)]
    pub command: ServiceCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServiceCommand {
    List(list::ListArgs),
    Start(start::StartArgs),
    Stop(stop::StopArgs),
    Restart(restart::RestartArgs),
    Status(status::StatusArgs),
    Logs(logs::LogsArgs),
    Enable(enable::EnableArgs),
    Disable(disable::DisableArgs),
}

pub async fn run(ctx: &AppContext, args: ServiceArgs) -> Result<()> {
    match args.command {
        ServiceCommand::List(args) => list::run(ctx, args).await,
        ServiceCommand::Start(args) => start::run(ctx, args).await,
        ServiceCommand::Stop(args) => stop::run(ctx, args).await,
        ServiceCommand::Restart(args) => restart::run(ctx, args).await,
        ServiceCommand::Status(args) => status::run(ctx, args).await,
        ServiceCommand::Logs(args) => logs::run(ctx, args).await,
        ServiceCommand::Enable(args) => enable::run(ctx, args).await,
        ServiceCommand::Disable(args) => disable::run(ctx, args).await,
    }
}
