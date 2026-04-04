use anyhow::{Context, Result};
use clap::Args;

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct InfoArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: InfoArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let command = format!(
        "printf 'NIXOS_VERSION\n'; nixos-version 2>/dev/null; printf '\nKERNEL\n'; uname -r 2>/dev/null; printf '\nSTATE_VERSION\n'; awk -F '\"' '/system.stateVersion/ {{print $2; exit}}' {cfg}/configuration.nix 2>/dev/null; printf '\nARCH\n'; uname -m 2>/dev/null; printf '\nHOST\n'; hostname 2>/dev/null; printf '\nUPTIME\n'; uptime -p 2>/dev/null || uptime 2>/dev/null; printf '\nNIX_VERSION\n'; nix --version 2>/dev/null",
        cfg = crate::commands::shell_quote(&machine.config_dir)
    );
    let output = crate::exec::run(&machine, &command).await?;
    if !output.success() {
        anyhow::bail!("couldn't load system info: {}", output.stderr.trim());
    }

    let mut section = "";
    let mut info = std::collections::BTreeMap::new();
    for line in output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        match line {
            "NIXOS_VERSION" | "KERNEL" | "STATE_VERSION" | "ARCH" | "HOST" | "UPTIME"
            | "NIX_VERSION" => {
                section = line;
            }
            _ if !section.is_empty() => {
                info.insert(section, line.to_string());
            }
            _ => {}
        }
    }

    ctx.output.info(&format!("system info — {}", machine.name));
    ctx.output.blank();
    ctx.output.kv(
        "nixos version",
        &info.get("NIXOS_VERSION").context("missing nixos version")?,
    );
    ctx.output
        .kv("kernel", &info.get("KERNEL").context("missing kernel")?);
    ctx.output.kv(
        "state version",
        info.get("STATE_VERSION").unwrap_or(&"unknown".to_string()),
    );
    ctx.output.kv(
        "architecture",
        &info.get("ARCH").context("missing architecture")?,
    );
    ctx.output
        .kv("hostname", &info.get("HOST").context("missing hostname")?);
    ctx.output
        .kv("uptime", &info.get("UPTIME").context("missing uptime")?);
    ctx.output.kv(
        "nix version",
        &info.get("NIX_VERSION").context("missing nix version")?,
    );
    Ok(())
}
