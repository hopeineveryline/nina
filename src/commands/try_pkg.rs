use anyhow::Result;
use clap::Args;

use crate::commands::{package_shell_ref, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct TryPkgArgs {
    pub packages: Vec<String>,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: TryPkgArgs) -> Result<()> {
    if args.packages.is_empty() {
        ctx.output
            .face("give me at least one package, like: nina try ripgrep ♡");
        return Ok(());
    }

    let joined = args
        .packages
        .iter()
        .map(|p| format!("nixpkgs#{}", package_shell_ref(p)))
        .collect::<Vec<_>>()
        .join(" ");
    let cmd = format!("nix shell {}", joined);
    run_machine_command(
        ctx,
        &args.on,
        "opening a temporary shell",
        &cmd,
        "try",
        false,
    )
    .await
}
