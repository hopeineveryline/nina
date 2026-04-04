use anyhow::Result;
use clap::Args;

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct InfoArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: InfoArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let output = crate::exec::run(
        &machine,
        "printf 'SIZE\n'; du -sh /nix/store 2>/dev/null | cut -f1; printf '\nTOTAL\n'; find /nix/store -mindepth 1 -maxdepth 1 | wc -l; printf '\nLIVE\n'; nix path-info -r /run/current-system 2>/dev/null | wc -l",
    )
    .await?;
    if !output.success() {
        anyhow::bail!("couldn't inspect the nix store: {}", output.stderr.trim());
    }

    let mut section = "";
    let mut size = String::new();
    let mut total = 0_u64;
    let mut live = 0_u64;
    for line in output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        match line {
            "SIZE" | "TOTAL" | "LIVE" => section = line,
            _ => match section {
                "SIZE" if size.is_empty() => size = line.to_string(),
                "TOTAL" if total == 0 => total = line.parse().unwrap_or(0),
                "LIVE" if live == 0 => live = line.parse().unwrap_or(0),
                _ => {}
            },
        }
    }
    let dead = total.saturating_sub(live);
    ctx.output.info(&format!("nix store on {}", machine.name));
    ctx.output.blank();
    ctx.output.kv("location", "/nix/store");
    ctx.output.kv("disk used", &size);
    ctx.output.kv("store paths", &total.to_string());
    ctx.output.kv("live paths", &live.to_string());
    ctx.output.kv("dead paths", &dead.to_string());
    ctx.output.blank();
    ctx.output.tip("run 'nina clean' to free up dead paths  ♡");
    Ok(())
}
