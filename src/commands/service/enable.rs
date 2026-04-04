use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct EnableArgs {
    pub service: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: EnableArgs) -> Result<()> {
    let command = format!(
        "sudo systemctl enable {}",
        crate::commands::shell_quote(&args.service)
    );
    run_machine_command(
        ctx,
        &args.on,
        "enabling service",
        &command,
        "service-enable",
        false,
    )
    .await
}
