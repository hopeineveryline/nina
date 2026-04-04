use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct RestartArgs {
    pub service: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: RestartArgs) -> Result<()> {
    let command = format!(
        "sudo systemctl restart {}",
        crate::commands::shell_quote(&args.service)
    );
    run_machine_command(
        ctx,
        &args.on,
        "restarting service",
        &command,
        "service-restart",
        false,
    )
    .await
}
