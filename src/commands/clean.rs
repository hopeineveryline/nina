use anyhow::Result;
use clap::Args;

use crate::commands::{confirm_action, AppContext};

#[derive(Debug, Clone, Args)]
pub struct CleanArgs {
    #[arg(long)]
    pub on: Option<String>,
    #[arg(long)]
    pub all: bool,
}

pub async fn run(ctx: &AppContext, args: CleanArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let before = collect_cleanup_stats(&machine).await?;
    let remove_count = if args.all {
        before.generation_count.saturating_sub(1)
    } else {
        before
            .generation_count
            .saturating_sub(ctx.config.generations)
    };

    ctx.output
        .info(&format!("cleaning generations on {}...", machine.name));
    ctx.output.step(&format!(
        "you have {} generations and {} in /nix/store",
        before.generation_count, before.store_size
    ));
    if args.all {
        ctx.output.step(&format!(
            "this will remove about {} old generation(s), keep the current one, and run garbage collection.",
            remove_count
        ));
    } else {
        ctx.output.step(&format!(
            "keeping the latest {} generations means removing {} older generation(s).",
            ctx.config.generations, remove_count
        ));
    }

    if !confirm_action(ctx.config.confirm, "continue with cleanup?")? {
        ctx.output.happy("okay, cancelled with no cleanup ♡");
        return Ok(());
    }

    let command = if args.all {
        "sudo nix-collect-garbage -d".to_string()
    } else {
        format!(
            "keep={} && gens=$(nix-env --list-generations -p /nix/var/nix/profiles/system | awk '{{print $1}}' | sort -n) && count=$(printf '%s\n' \"$gens\" | grep -c .) && if [ \"$count\" -gt \"$keep\" ]; then delete_count=$((count-keep)); for gen in $(printf '%s\n' \"$gens\" | head -n \"$delete_count\"); do sudo nix-env --delete-generations -p /nix/var/nix/profiles/system \"$gen\"; done; fi && sudo nix-collect-garbage",
            ctx.config.generations
        )
    };

    let result = crate::exec::run(&machine, &command).await?;
    if !result.success() {
        anyhow::bail!("cleanup failed: {}", result.stderr.trim());
    }

    let after = collect_cleanup_stats(&machine).await?;
    let freed = before.store_bytes.saturating_sub(after.store_bytes);
    ctx.output.success("cleanup done");
    ctx.output.step(&format!(
        "removed {} generation(s), leaving {} generation(s), and freed about {} bytes.",
        before
            .generation_count
            .saturating_sub(after.generation_count),
        after.generation_count,
        freed
    ));
    Ok(())
}

struct CleanupStats {
    generation_count: u32,
    store_size: String,
    store_bytes: u64,
}

async fn collect_cleanup_stats(machine: &crate::machine::Machine) -> Result<CleanupStats> {
    let generation_count = crate::exec::run(
        machine,
        "nix-env --list-generations -p /nix/var/nix/profiles/system | grep -c .",
    )
    .await?;
    let store_bytes =
        crate::exec::run(machine, "du -sb /nix/store 2>/dev/null | cut -f1 || echo 0").await?;
    let store_size =
        crate::exec::run(machine, "du -sh /nix/store 2>/dev/null | cut -f1 || echo 0").await?;

    Ok(CleanupStats {
        generation_count: generation_count.stdout.trim().parse().unwrap_or(0),
        store_size: store_size.stdout.trim().to_string(),
        store_bytes: store_bytes.stdout.trim().parse().unwrap_or(0),
    })
}
