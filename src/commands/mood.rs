use anyhow::{Context, Result};
use clap::Args;
use tokio::task::JoinSet;

use crate::commands::AppContext;
use crate::machine::Machine;

#[derive(Debug, Clone, Args)]
pub struct MoodArgs {}

pub async fn run(ctx: &AppContext, _args: MoodArgs) -> Result<()> {
    if ctx.config.machines.is_empty() {
        ctx.output
            .face("i'm cozy, but i need at least one machine in ~/.nina.conf ♡");
        return Ok(());
    }

    let mut tasks = JoinSet::new();
    for raw in &ctx.config.machines {
        let machine = Machine::from_config(raw);
        tasks.spawn(async move { crate::commands::status::collect_status(machine).await });
    }

    let mut rows = Vec::new();
    while let Some(result) = tasks.join_next().await {
        rows.push(result.context("mood status task crashed")??);
    }
    rows.sort_by(|a, b| a.machine.cmp(&b.machine));

    let all_ok = rows.iter().all(|r| r.ok);
    let vibes = rows
        .into_iter()
        .map(|row| {
            let vibe = if !row.ok {
                "needs a little care right now"
            } else if row.disk.contains('G') || row.disk.contains('M') {
                "is happy and healthy"
            } else {
                "is doing great"
            };
            format!("{} is {}", row.machine, vibe)
        })
        .collect::<Vec<_>>();

    if all_ok {
        ctx.output.cozy(&format!("{}~ 🍡", vibes.join(" · ")));
    } else {
        ctx.output.cozy(&format!("{}~", vibes.join(" · ")));
    }
    Ok(())
}
