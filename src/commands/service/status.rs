use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct StatusArgs {
    pub service: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: StatusArgs) -> Result<()> {
    let command = format!(
        "systemctl status {} --no-pager",
        crate::commands::shell_quote(&args.service)
    );
    run_machine_command(
        ctx,
        &args.on,
        "checking service status",
        &command,
        "service-status",
        false,
    )
    .await
}
