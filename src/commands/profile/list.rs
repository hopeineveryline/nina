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
        "listing profile packages",
        "nix profile list",
        "profile-list",
        false,
    )
    .await
}
