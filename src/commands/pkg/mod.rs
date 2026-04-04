pub mod closure;
pub mod deps;
pub mod path;
pub mod size;
pub mod why;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct PkgArgs {
    #[command(subcommand)]
    pub command: PkgCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum PkgCommand {
    Why(why::WhyArgs),
    Deps(deps::DepsArgs),
    Size(size::SizeArgs),
    Path(path::PathArgs),
    Closure(closure::ClosureArgs),
}

pub async fn run(ctx: &AppContext, args: PkgArgs) -> Result<()> {
    match args.command {
        PkgCommand::Why(args) => why::run(ctx, args).await,
        PkgCommand::Deps(args) => deps::run(ctx, args).await,
        PkgCommand::Size(args) => size::run(ctx, args).await,
        PkgCommand::Path(args) => path::run(ctx, args).await,
        PkgCommand::Closure(args) => closure::run(ctx, args).await,
    }
}
