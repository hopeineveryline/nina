use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct UpgradeArgs {
    #[arg(long)]
    pub on: Option<String>,
    #[arg(long)]
    pub check: bool,
}

pub async fn run(ctx: &AppContext, args: UpgradeArgs) -> Result<()> {
    run_machine_command(
        ctx,
        &args.on,
        "updating channels",
        "sudo nix-channel --update",
        "upgrade",
        true,
    )
    .await?;

    let machine = ctx.machine(&args.on)?;
    let cmd = if args.check {
        format!(
            "sudo nixos-rebuild build -I nixos-config={}/configuration.nix",
            machine.config_dir
        )
    } else {
        format!(
            "sudo nixos-rebuild switch -I nixos-config={}/configuration.nix",
            machine.config_dir
        )
    };

    run_machine_command(ctx, &args.on, "upgrading your system", &cmd, "upgrade", true).await
}
