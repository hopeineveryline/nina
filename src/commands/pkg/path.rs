use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct PathArgs {
    pub package: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: PathArgs) -> Result<()> {
    let command = format!(
        "nix eval --raw {}",
        crate::commands::shell_quote(&format!("nixpkgs#{}.outPath", args.package))
    );
    run_machine_command(
        ctx,
        &args.on,
        "resolving package store path",
        &command,
        "pkg-path",
        false,
    )
    .await
}
