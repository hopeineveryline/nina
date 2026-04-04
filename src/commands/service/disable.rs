use anyhow::Result;
use clap::Args;

use crate::commands::{confirm_action, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct DisableArgs {
    pub service: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: DisableArgs) -> Result<()> {
    if !confirm_action(ctx.config.confirm, "disable this service at boot?")? {
        ctx.output.warn("okay, cancelled with no changes ♡");
        return Ok(());
    }
    let command = format!(
        "sudo systemctl disable {}",
        crate::commands::shell_quote(&args.service)
    );
    run_machine_command(
        ctx,
        &args.on,
        "disabling service",
        &command,
        "service-disable",
        false,
    )
    .await
}
