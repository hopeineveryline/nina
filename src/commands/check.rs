use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct CheckArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: CheckArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let cmd = format!(
        "sudo nixos-rebuild build -I nixos-config={}/configuration.nix",
        machine.config_dir
    );
    run_machine_command(ctx, &args.on, "checking config", &cmd, "check", false).await
}
