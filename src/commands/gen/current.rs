use anyhow::Result;
use clap::Args;

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct CurrentArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: CurrentArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let output = crate::exec::run(
        &machine,
        "nix-env --list-generations -p /nix/var/nix/profiles/system | awk '/\\(current\\)/ {print $1}'",
    )
    .await?;
    if !output.success() {
        anyhow::bail!(
            "couldn't find the current generation: {}",
            output.stderr.trim()
        );
    }
    ctx.output.print(&output.stdout.trim());
    Ok(())
}
