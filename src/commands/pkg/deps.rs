use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct DepsArgs {
    pub package: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: DepsArgs) -> Result<()> {
    let command = format!(
        "nix-store --query --requisites $(nix eval --raw {})",
        crate::commands::shell_quote(&format!("nixpkgs#{}.outPath", args.package))
    );
    run_machine_command(
        ctx,
        &args.on,
        "peeking at what this package needs",
        &command,
        "pkg-deps",
        false,
    )
    .await
}
