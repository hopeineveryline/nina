use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct UpdateArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: UpdateArgs) -> Result<()> {
    run_machine_command(
        ctx,
        &args.on,
        "freshening channels",
        "sudo nix-channel --update",
        "update",
        true,
    )
    .await
}
