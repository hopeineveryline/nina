use anyhow::{Context, Result};
use clap::Args;
use tokio::task::JoinSet;

use crate::commands::AppContext;
use crate::machine::Machine;

#[derive(Debug, Clone, Args)]
pub struct StatusArgs {
    #[arg(long)]
    pub on: Option<String>,
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct MachineStatus {
    pub(crate) machine: String,
    pub(crate) ok: bool,
    pub(crate) generation: String,
    pub(crate) generation_count: String,
    pub(crate) uptime: String,
    pub(crate) last_applied: String,
    pub(crate) channel: String,
    pub(crate) dirty: String,
    pub(crate) disk: String,
    pub(crate) error: Option<String>,
}

pub async fn run(ctx: &AppContext, args: StatusArgs) -> Result<()> {
    if args.all {
        ctx.output.info("checking all your machines...");

        let mut tasks = JoinSet::new();
        for raw in &ctx.config.machines {
            let machine = Machine::from_config(raw);
            tasks.spawn(async move { collect_status(machine).await });
        }

        let mut rows = Vec::new();
        while let Some(result) = tasks.join_next().await {
            rows.push(result.context("status task crashed")??);
        }
        rows.sort_by(|a, b| a.machine.cmp(&b.machine));

        ctx.output.print_muted(
            "  machine   status    generation   last-applied        channel         dirty  disk",
        );
        ctx.output.print_muted(
            "  ───────   ──────    ──────────   ────────────────    ─────────────  ─────  ────",
        );
        for row in &rows {
            let generation = if row.generation.is_empty() {
                "?"
            } else {
                &row.generation
            };
            let last_applied = if row.last_applied.is_empty() {
                "?"
            } else {
                &row.last_applied
            };
            let channel = if row.channel.is_empty() {
                "?"
            } else {
                &row.channel
            };
            let disk = if row.disk.is_empty() { "?" } else { &row.disk };
            ctx.output.print(&format!(
                "  {:<7}  gen {:<6}  {:<16}  {:<12}  {}",
                row.machine, generation, last_applied, channel, disk
            ));
            if let Some(error) = &row.error {
                ctx.output.warn(&format!("↳ {}", error));
            }
        }

        if rows.iter().all(|row| row.ok) {
            ctx.output.happy("everything looks great ♡");
        } else {
            ctx.output
                .warn("at least one machine needs attention, but the summary is above.");
        }
        return Ok(());
    }

    let machine = ctx.machine(&args.on)?;
    let status = collect_status(machine).await?;
    if !status.ok {
        if let Some(error) = status.error {
            ctx.output.error(&error);
        }
        anyhow::bail!("status check failed");
    }

    ctx.output.info(&format!("status for {}", status.machine));
    ctx.output.kv("generation", &status.generation);
    ctx.output.kv("total gens", &status.generation_count);
    ctx.output.kv("last apply", &status.last_applied);
    ctx.output.kv("channel", &status.channel);
    ctx.output.kv("dirty", &status.dirty);
    ctx.output.kv("uptime", &status.uptime);
    ctx.output.kv("/nix/store", &status.disk);
    Ok(())
}

pub(crate) async fn collect_status(machine: Machine) -> Result<MachineStatus> {
    let command = format!(
        "printf 'UPTIME\n'; uptime; printf '\nGEN\n'; nix-env --list-generations -p /nix/var/nix/profiles/system | awk '/\\(current\\)/ {{print}}'; printf '\nCOUNT\n'; nix-env --list-generations -p /nix/var/nix/profiles/system | grep -c .; printf '\nLAST\n'; stat -c %y /nix/var/nix/profiles/system 2>/dev/null | cut -d. -f1; printf '\nCHANNEL\n'; nix-channel --list | head -n1 | awk '{{print $2}}'; printf '\nDIRTY\n'; if [ {cfg}/configuration.nix -nt /nix/var/nix/profiles/system ]; then echo modified-since-apply; else echo clean; fi; printf '\nDISK\n'; du -sh /nix/store 2>/dev/null | cut -f1",
        cfg = machine.config_dir
    );
    let result = crate::exec::run(&machine, &command).await;

    match result {
        Ok(output) if output.success() => Ok(parse_status_output(&machine.name, &output.stdout)),
        Ok(output) => Ok(MachineStatus {
            machine: machine.name,
            ok: false,
            generation: String::new(),
            generation_count: String::new(),
            uptime: String::new(),
            last_applied: String::new(),
            channel: String::new(),
            dirty: String::new(),
            disk: String::new(),
            error: Some(output.stderr.trim().to_string()),
        }),
        Err(err) => Ok(MachineStatus {
            machine: machine.name,
            ok: false,
            generation: String::new(),
            generation_count: String::new(),
            uptime: String::new(),
            last_applied: String::new(),
            channel: String::new(),
            dirty: String::new(),
            disk: String::new(),
            error: Some(err.to_string()),
        }),
    }
}

fn parse_status_output(machine: &str, stdout: &str) -> MachineStatus {
    let mut mode = "";
    let mut uptime = String::new();
    let mut generation = String::new();
    let mut generation_count = String::new();
    let mut last_applied = String::new();
    let mut channel = String::new();
    let mut dirty = String::new();
    let mut disk = String::new();

    for raw in stdout.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        match line {
            "UPTIME" | "GEN" | "COUNT" | "LAST" | "CHANNEL" | "DIRTY" | "DISK" => {
                mode = line;
            }
            _ => match mode {
                "UPTIME" if uptime.is_empty() => uptime = line.to_string(),
                "GEN" if generation.is_empty() => {
                    generation = line.split_whitespace().next().unwrap_or(line).to_string()
                }
                "COUNT" if generation_count.is_empty() => generation_count = line.to_string(),
                "LAST" if last_applied.is_empty() => last_applied = line.to_string(),
                "CHANNEL" if channel.is_empty() => channel = line.to_string(),
                "DIRTY" if dirty.is_empty() => dirty = line.to_string(),
                "DISK" if disk.is_empty() => disk = line.to_string(),
                _ => {}
            },
        }
    }

    MachineStatus {
        machine: machine.to_string(),
        ok: true,
        generation,
        generation_count,
        uptime,
        last_applied,
        channel,
        dirty,
        disk,
        error: None,
    }
}
