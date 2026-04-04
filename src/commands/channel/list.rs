use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: ListArgs) -> Result<()> {
    run_machine_command(
        ctx,
        &args.on,
        "listing channels",
        "nix-channel --list",
        "channel-list",
        false,
    )
    .await
}
