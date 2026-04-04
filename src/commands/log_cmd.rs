use anyhow::Result;
use clap::Args;

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct LogCmdArgs {
    #[arg(long, default_value_t = 10)]
    pub last: usize,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: LogCmdArgs) -> Result<()> {
    let entries = crate::log::read_last(args.last)?;
    if entries.is_empty() {
        ctx.output.face("no nina operations logged yet.");
        return Ok(());
    }

    ctx.output
        .print_muted("ts                      machine   command   outcome   duration");
    ctx.output
        .print_muted("──────────────────────  ───────   ───────   ───────   ────────");
    for row in entries {
        if let Some(on) = &args.on {
            if &row.machine != on {
                continue;
            }
        }
        ctx.output.print(&format!(
            "{:<22}  {:<7}   {:<7}   {:<7}   {} ms",
            row.ts.format("%Y-%m-%d %H:%M:%S"),
            row.machine,
            row.command,
            row.outcome,
            row.duration_ms
        ));
    }
    Ok(())
}
