use anyhow::Result;
use clap::Args;

use crate::commands::{confirm_action, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct DeleteArgs {
    pub generation: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: DeleteArgs) -> Result<()> {
    if !confirm_action(ctx.config.confirm, "delete this generation?")? {
        ctx.output.warn("okay, cancelled with no changes ♡");
        return Ok(());
    }
    let command = format!(
        "nix-env --delete-generations -p /nix/var/nix/profiles/system {}",
        args.generation
    );
    run_machine_command(
        ctx,
        &args.on,
        "deleting generation",
        &command,
        "gen-delete",
        false,
    )
    .await
}
