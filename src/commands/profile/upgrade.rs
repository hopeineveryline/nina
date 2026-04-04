use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct UpgradeArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: UpgradeArgs) -> Result<()> {
    run_machine_command(
        ctx,
        &args.on,
        "upgrading profile packages",
        "nix profile upgrade '.*'",
        "profile-upgrade",
        false,
    )
    .await
}
