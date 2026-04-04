pub mod gc;
pub mod info;
pub mod path;
pub mod repair;
pub mod verify;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct StoreArgs {
    #[command(subcommand)]
    pub command: StoreCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum StoreCommand {
    Gc(gc::GcArgs),
    Verify(verify::VerifyArgs),
    Repair(repair::RepairArgs),
    Info(info::InfoArgs),
    Path(path::PathArgs),
}

pub async fn run(ctx: &AppContext, args: StoreArgs) -> Result<()> {
    match args.command {
        StoreCommand::Gc(args) => gc::run(ctx, args).await,
        StoreCommand::Verify(args) => verify::run(ctx, args).await,
        StoreCommand::Repair(args) => repair::run(ctx, args).await,
        StoreCommand::Info(args) => info::run(ctx, args).await,
        StoreCommand::Path(args) => path::run(ctx, args).await,
    }
}
