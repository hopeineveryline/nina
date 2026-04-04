use anyhow::Result;
use clap::Args;

use crate::commands::{current_dir_command, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct CloneArgs {
    pub url: String,
}

pub async fn run(ctx: &AppContext, args: CloneArgs) -> Result<()> {
    let command = current_dir_command(&format!(
        "nix flake clone {}",
        crate::commands::shell_quote(&args.url)
    ))?;
    let on = None;
    run_machine_command(
        ctx,
        &on,
        "cloning flake repository",
        &command,
        "flake-clone",
        false,
    )
    .await
}
