use crate::commands::AppContext;
use crate::machine::Machine;
use anyhow::Result;
use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct HelloArgs {}

pub async fn run(ctx: &AppContext, _args: HelloArgs) -> Result<()> {
    ctx.output.face("hi! i'm nina~");
    ctx.output.print_muted("i'm your calm little nix helper.");
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

    if ctx.config.animate {
        ctx.output.excited("i hope you'll like these ♡");
    }

    ctx.output.blank();
    ctx.output
        .print_muted("try 'nina help' to see everything i can do ♡");
    Ok(())
}
