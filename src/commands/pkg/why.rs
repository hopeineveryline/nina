use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct WhyArgs {
    pub package: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: WhyArgs) -> Result<()> {
    let command = format!(
        "nix why-depends /run/current-system {}",
        crate::commands::shell_quote(&format!("nixpkgs#{}", args.package))
    );
    run_machine_command(
        ctx,
        &args.on,
        "tracing why this package snuck in",
        &command,
        "pkg-why",
        false,
    )
    .await
}
