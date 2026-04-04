use anyhow::Result;
use clap::Args;

use crate::commands::{current_dir_command, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct InitArgs {}

pub async fn run(ctx: &AppContext, _args: InitArgs) -> Result<()> {
    let command = current_dir_command("nix flake init")?;
    let on = None;
    run_machine_command(
        ctx,
        &on,
        "creating a new flake",
        &command,
        "flake-init",
        false,
    )
    .await
}
