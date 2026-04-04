use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct BootArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: BootArgs) -> Result<()> {
    run_machine_command(
        ctx,
        &args.on,
        "checking boot entries",
        "bootctl list",
        "boot",
        false,
    )
    .await
}
