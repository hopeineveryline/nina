use anyhow::Result;
use clap::Args;

use crate::commands::{current_dir_command, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct PinArgs {
    pub input: String,
    pub commit: Option<String>,
    #[arg(long)]
    pub stable: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UnpinArgs {
    pub input: String,
}

pub async fn run(ctx: &AppContext, args: PinArgs) -> Result<()> {
    let command = if args.stable {
        current_dir_command(&format!(
            "rev=$(nix eval --raw --expr {} ) && nix flake lock --override-input {} github:NixOS/nixpkgs/$rev",
            crate::commands::shell_quote("let flake = builtins.getFlake \"github:NixOS/nixpkgs/nixos-25.05\"; in flake.sourceInfo.rev or flake.rev"),
            crate::commands::shell_quote(&args.input),
        ))?
    } else {
        let commit = args
            .commit
            .ok_or_else(|| anyhow::anyhow!("please provide a commit or use --stable"))?;
        current_dir_command(&format!(
            "nix flake lock --override-input {} {}",
            crate::commands::shell_quote(&args.input),
            crate::commands::shell_quote(&format!("github:NixOS/nixpkgs/{}", commit))
        ))?
    };
    let on = None;
    run_machine_command(ctx, &on, "pinning flake input", &command, "pin", false).await
}

pub async fn run_unpin(ctx: &AppContext, args: UnpinArgs) -> Result<()> {
    let command = current_dir_command(&format!(
        "nix flake update --update-input {}",
        crate::commands::shell_quote(&args.input)
    ))?;
    let on = None;
    run_machine_command(ctx, &on, "unpinning flake input", &command, "unpin", false).await
}
