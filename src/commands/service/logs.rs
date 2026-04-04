use anyhow::Result;
use clap::Args;

use crate::commands::{run_attached_machine_command, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct LogsArgs {
    pub service: String,
    #[arg(short = 'f', long)]
    pub follow: bool,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: LogsArgs) -> Result<()> {
    let command = if args.follow {
        format!(
            "journalctl -u {} -f",
            crate::commands::shell_quote(&args.service)
        )
    } else {
        format!(
            "journalctl -u {} -n 50 --no-pager",
            crate::commands::shell_quote(&args.service)
        )
    };
    if args.follow {
        return run_attached_machine_command(
            ctx,
            &args.on,
            "following service logs",
            &command,
            "service-logs",
            false,
        )
        .await;
    }
    run_machine_command(
        ctx,
        &args.on,
        "showing service logs",
        &command,
        "service-logs",
        false,
    )
    .await
}
