use anyhow::Result;
use clap::Args;

use crate::commands::{current_dir_command, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct CheckArgs {}

pub async fn run(ctx: &AppContext, _args: CheckArgs) -> Result<()> {
    let command = current_dir_command("nix flake check")?;
    let on = None;
    run_machine_command(
        ctx,
        &on,
        "checking flake outputs",
        &command,
        "flake-check",
        false,
    )
    .await
}
