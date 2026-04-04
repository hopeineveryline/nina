use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct InstallArgs {
    pub package: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: InstallArgs) -> Result<()> {
    let command = format!(
        "nix profile install {}",
        crate::commands::shell_quote(&format!("nixpkgs#{}", args.package))
    );
    run_machine_command(
        ctx,
        &args.on,
        "installing profile package",
        &command,
        "profile-install",
        false,
    )
    .await
}
