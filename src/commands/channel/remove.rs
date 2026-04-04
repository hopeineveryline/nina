use anyhow::Result;
use clap::Args;

use crate::commands::{confirm_action, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct RemoveArgs {
    pub name: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: RemoveArgs) -> Result<()> {
    if !confirm_action(ctx.config.confirm, "remove this channel?")? {
        ctx.output.warn("okay, cancelled with no changes ♡");
        return Ok(());
    }
    let command = format!(
        "sudo nix-channel --remove {}",
        crate::commands::shell_quote(&args.name)
    );
    run_machine_command(
        ctx,
        &args.on,
        "removing channel",
        &command,
        "channel-remove",
        false,
    )
    .await
}
