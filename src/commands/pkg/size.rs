use anyhow::Result;
use clap::Args;

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct SizeArgs {
    pub package: String,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: SizeArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let package_ref = crate::commands::shell_quote(&format!("nixpkgs#{}", args.package));
    let solo = crate::exec::run(&machine, &format!("nix path-info -Sh {}", package_ref)).await?;
    let closure =
        crate::exec::run(&machine, &format!("nix path-info -rSh {}", package_ref)).await?;
    if !solo.success() || !closure.success() {
        anyhow::bail!("couldn't inspect package sizes");
    }
    ctx.output.info(&format!("size of {}", args.package));
    ctx.output.blank();
    ctx.output.kv("package", solo.stdout.trim());
    ctx.output.kv("full closure", closure.stdout.trim());
    Ok(())
}
