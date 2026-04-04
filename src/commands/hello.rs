use anyhow::Result;
use clap::Args;
use std::io::IsTerminal;
use tokio::time::{sleep, Duration};

use crate::commands::AppContext;
use crate::machine::Machine;

#[derive(Debug, Clone, Args)]
pub struct HelloArgs {}

pub async fn run(ctx: &AppContext, _args: HelloArgs) -> Result<()> {
    ctx.output.face("hi! i'm nina~");
    ctx.output.print_muted("i'm your friendly nix helper");
    ctx.output
        .print_muted("i love declarative systems and calm terminal vibes ♡");
    ctx.output.blank();

    ctx.output.print_muted("i'm managing:");
    for raw in &ctx.config.machines {
        let machine = Machine::from_config(raw);
        let default_tag = if raw.default { ", default" } else { "" };
        ctx.output.print(&format!(
            "  - {} ({}{default_tag})",
            machine.name,
            machine.endpoint_label()
        ));
    }

    if ctx.config.animate && std::io::stdout().is_terminal() {
        let _ = crate::dango::DangoPlayer::play_once(crate::dango::DangoAnimation::Dance, (1, 1));
        sleep(Duration::from_millis(800)).await;
    }

    ctx.output.blank();
    ctx.output
        .print_muted("try 'nina help' to see everything i can do ♡");
    Ok(())
}
