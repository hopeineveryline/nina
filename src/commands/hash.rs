use anyhow::{Context, Result};
use clap::Args;

use crate::commands::{copy_to_clipboard, current_dir_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct HashArgs {
    pub path: String,
}

pub async fn run(ctx: &AppContext, args: HashArgs) -> Result<()> {
    let command = current_dir_command(&format!(
        "nix hash path {}",
        crate::commands::shell_quote(&args.path)
    ))?;
    let output = crate::exec::run_local(&command, |_, _| {}).await?;
    if !output.success() {
        anyhow::bail!("hashing failed: {}", output.stderr.trim());
    }
    let hash = output
        .stdout
        .lines()
        .find(|line| !line.trim().is_empty())
        .context("missing hash output")?
        .trim()
        .to_string();
    let _ = copy_to_clipboard(&hash);
    ctx.output.info("hash ready");
    ctx.output.blank();
    ctx.output.kv_succ(&hash, "(copied to clipboard)");
    ctx.output.blank();
    Ok(())
}
