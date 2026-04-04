use anyhow::Result;
use clap::Args;

use crate::commands::{current_dir_command, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct ShowArgs {}

pub async fn run(ctx: &AppContext, _args: ShowArgs) -> Result<()> {
    let command = current_dir_command("nix flake show")?;
    let on = None;
    run_machine_command(
        ctx,
        &on,
        "showing flake outputs",
        &command,
        "flake-show",
        false,
    )
    .await
}
