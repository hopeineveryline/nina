use anyhow::Result;
use clap::Args;

use crate::commands::{confirm_action, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct RepairArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: RepairArgs) -> Result<()> {
    if !confirm_action(ctx.config.confirm, "repair corrupted store paths?")? {
        ctx.output.warn("okay, cancelled with no changes ♡");
        return Ok(());
    }
    run_machine_command(
        ctx,
        &args.on,
        "repairing store paths",
        "nix store repair --all",
        "store-repair",
        false,
    )
    .await
}
