use anyhow::Result;
use clap::Args;

use crate::commands::{run_attached_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct ReplArgs {
    #[arg(long)]
    pub pure: bool,
}

pub async fn run(ctx: &AppContext, args: ReplArgs) -> Result<()> {
    ctx.output.tip("type pkgs.<name> to inspect any package");
    ctx.output.tip("type :q to exit  ♡");
    ctx.output.blank();

    let command = if args.pure {
        "nix repl".to_string()
    } else {
        "nix repl --expr 'import <nixpkgs> {}'".to_string()
    };
    let on = None;
    run_attached_machine_command(ctx, &on, "opening nix repl", &command, "repl", false).await
}
