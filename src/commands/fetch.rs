use anyhow::{Context, Result};
use clap::Args;

use crate::commands::{copy_to_clipboard, AppContext};

#[derive(Debug, Clone, Args)]
pub struct FetchArgs {
    pub url: String,
}

pub async fn run(ctx: &AppContext, args: FetchArgs) -> Result<()> {
    ctx.output.info("fetching...");
    let command = format!(
        "nix-prefetch-url --print-path {}",
        crate::commands::shell_quote(&args.url)
    );
    let output = crate::exec::run_local(&command, |_, _| {}).await?;
    if !output.success() {
        anyhow::bail!("fetch failed: {}", output.stderr.trim());
    }

    let mut lines = output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let hash = lines.next().context("missing fetched hash")?.to_string();
    let store_path = lines.next().unwrap_or_default().to_string();
    let size_output = if store_path.is_empty() {
        None
    } else {
        Some(
            crate::exec::run_local(
                &format!(
                    "du -sh {} | cut -f1",
                    crate::commands::shell_quote(&store_path)
                ),
                |_, _| {},
            )
            .await?,
        )
    };
    let size = size_output
        .as_ref()
        .filter(|output| output.success())
        .map(|output| output.stdout.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let _ = copy_to_clipboard(&hash);
    ctx.output.kv("hash", &hash);
    ctx.output.kv("size", &size);
    ctx.output.happy("(hash copied to clipboard)");
    Ok(())
}
