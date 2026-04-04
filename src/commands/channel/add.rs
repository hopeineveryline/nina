use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct AddArgs {
    pub url: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: AddArgs) -> Result<()> {
    let command = format!(
        "sudo nix-channel --add {}",
        crate::commands::shell_quote(&args.url)
    );
    run_machine_command(
        ctx,
        &args.on,
        "adding channel",
        &command,
        "channel-add",
        true,
    )
    .await
}
