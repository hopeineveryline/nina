use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct ApplyArgs {
    #[arg(long)]
    pub on: Option<String>,
    #[arg(long)]
    pub dry: bool,
    #[arg(long)]
    pub check: bool,
}

pub async fn run(ctx: &AppContext, args: ApplyArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let config = format!("{}/configuration.nix", machine.config_dir);

    let cmd = if args.dry {
        format!("sudo nixos-rebuild dry-activate -I nixos-config={config}")
    } else if args.check {
        format!("sudo nixos-rebuild build -I nixos-config={config}")
    } else {
        format!("sudo nixos-rebuild switch -I nixos-config={config}")
    };

    let needs_confirmation = !args.dry && !args.check;

    run_machine_command(
        ctx,
        &args.on,
        "putting your config live",
        &cmd,
        "apply",
        needs_confirmation,
    )
    .await
}
