use anyhow::Result;
use clap::Args;

use crate::commands::{confirm_action, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct RemoveArgs {
    pub package: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: RemoveArgs) -> Result<()> {
    if !confirm_action(ctx.config.confirm, "remove this profile package?")? {
        ctx.output.warn("okay, cancelled with no changes ♡");
        return Ok(());
    }
    let command = format!(
        "nix profile remove {}",
        crate::commands::shell_quote(&format!("nixpkgs#{}", args.package))
    );
    run_machine_command(
        ctx,
        &args.on,
        "removing profile package",
        &command,
        "profile-remove",
        false,
    )
    .await
}
