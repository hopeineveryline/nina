use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct GoArgs {
    pub generation: u32,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: GoArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let generations = crate::exec::run(
        &machine,
        "nix-env --list-generations -p /nix/var/nix/profiles/system | awk '{print $1}'",
    )
    .await?;
    if !generations.success()
        || !generations
            .stdout
            .split_whitespace()
            .any(|line| line == args.generation.to_string())
    {
        anyhow::bail!(
            "generation {} does not exist on {}",
            args.generation,
            machine.name
        );
    }

    let cmd = format!(
        "sudo nix-env --switch-generation {} -p /nix/var/nix/profiles/system && sudo /nix/var/nix/profiles/system/bin/switch-to-configuration switch",
        args.generation
    );
    run_machine_command(ctx, &args.on, "switching generations", &cmd, "go", true).await
}
