use anyhow::Result;
use clap::Args;

use crate::commands::{run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct DiffArgs {
    pub from: Option<u32>,
    pub to: Option<u32>,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: DiffArgs) -> Result<()> {
    let cmd = if let (Some(from), Some(to)) = (args.from, args.to) {
        format!(
            "nix store diff-closures /nix/var/nix/profiles/system-{from}-link /nix/var/nix/profiles/system-{to}-link"
        )
    } else {
        "prev=$(ls -d /nix/var/nix/profiles/system-*-link 2>/dev/null | sed 's#.*/system-##; s/-link$//' | sort -n | tail -n 2 | head -n 1) && nix store diff-closures /nix/var/nix/profiles/system-${prev}-link /nix/var/nix/profiles/system"
            .to_string()
    };

    run_machine_command(ctx, &args.on, "spotting the differences", &cmd, "diff", false).await
}
