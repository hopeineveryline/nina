use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct ClosureArgs {
    pub package: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: ClosureArgs) -> Result<()> {
    let command = format!(
        "nix path-info -rS {}",
        crate::commands::shell_quote(&format!("nixpkgs#{}", args.package))
    );
    run_machine_command(
        ctx,
        &args.on,
        "showing package closure",
        &command,
        "pkg-closure",
        false,
    )
    .await
}
