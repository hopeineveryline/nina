use anyhow::Result;
use clap::Args;

use crate::commands::{current_dir_command, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct LockArgs {}

pub async fn run(ctx: &AppContext, _args: LockArgs) -> Result<()> {
    let command = current_dir_command("nix flake lock")?;
    let on = None;
    run_machine_command(
        ctx,
        &on,
        "refreshing flake lock",
        &command,
        "flake-lock",
        false,
    )
    .await
}
