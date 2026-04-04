use anyhow::Result;
use clap::Args;

use crate::commands::{confirm_action, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct StopArgs {
    pub service: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: StopArgs) -> Result<()> {
    if !confirm_action(
        ctx.config.confirm,
        "this will interrupt the running service. continue?",
    )? {
        ctx.output.warn("okay, cancelled with no changes ♡");
        return Ok(());
    }
    let command = format!(
        "sudo systemctl stop {}",
        crate::commands::shell_quote(&args.service)
    );
    run_machine_command(
        ctx,
        &args.on,
        "stopping service",
        &command,
        "service-stop",
        false,
    )
    .await
}
