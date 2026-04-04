use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct VerifyArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: VerifyArgs) -> Result<()> {
    run_machine_command(
        ctx,
        &args.on,
        "checking the store over",
        "nix store verify --all",
        "store-verify",
        false,
    )
    .await
}
