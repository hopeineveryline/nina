use crate::commands::AppContext;
use crate::machine::Machine;
use anyhow::Result;
use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct HelloArgs {}

pub async fn run(ctx: &AppContext, _args: HelloArgs) -> Result<()> {
    ctx.output.print("(˶ᵔ ᵕ ᵔ˶)");
    ctx.output.blank();
    ctx.output.print_muted("hi, i'm nina.");
    ctx.output
        .print_muted("i make nix feel a little less like a foreign language.");
    ctx.output.blank();

    ctx.output.print_muted("machines i know:");
    for raw in &ctx.config.machines {
        let machine = Machine::from_config(raw);
        let default_tag = if raw.default { ", default" } else { "" };
        ctx.output.print(&format!(
            "  {}    {}{}",
            machine.name,
            machine.endpoint_label(),
            default_tag
        ));
    }

    ctx.output.blank();
    ctx.output.print_muted("'nina help' to get started.  ♡");
    Ok(())
}
