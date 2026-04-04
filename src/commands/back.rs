use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct BackArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: BackArgs) -> Result<()> {
    ctx.output.rollback("stepping back one generation...");
    run_machine_command(
        ctx,
        &args.on,
        "stepping back",
        "sudo nixos-rebuild switch --rollback",
        "back",
        true,
    )
    .await
}
