use anyhow::{Context, Result};
use clap::Args;
use tokio::task::JoinSet;

use crate::commands::AppContext;
use crate::machine::Machine;

#[derive(Debug, Clone, Args)]
pub struct DoctorArgs {
    #[arg(long)]
    pub on: Option<String>,
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Clone)]
struct DiagnosticRow {
    label: &'static str,
    ok: bool,
    detail: String,
    suggestion: Option<String>,
}

#[derive(Debug, Clone)]
struct DoctorReport {
    machine: String,
    rows: Vec<DiagnosticRow>,
}

pub async fn run(ctx: &AppContext, args: DoctorArgs) -> Result<()> {
    if args.all {
        let mut tasks = JoinSet::new();
        for raw in &ctx.config.machines {
            let machine = Machine::from_config(raw);
            let keep = ctx.config.generations;
            tasks.spawn(async move { gather_report(machine, keep).await });
        }

        while let Some(result) = tasks.join_next().await {
            render_report(ctx, &result.context("doctor task crashed")??);
        }
        return Ok(());
    }

    let machine = ctx.machine(&args.on)?;
    let report = gather_report(machine, ctx.config.generations).await?;
    render_report(ctx, &report);
    Ok(())
}

async fn gather_report(machine: Machine, keep_generations: u32) -> Result<DoctorReport> {
    let mut rows = Vec::new();

    rows.push(
        run_check(
            &machine,
            "channel health",
            "nix-channel --list",
            "no nix channel is configured",
            Some("run: nina update".to_string()),
        )
        .await,
    );

    rows.push(
        run_check(
            &machine,
            "nix daemon",
            "systemctl is-active nix-daemon",
            "nix-daemon is not active",
            Some("check: systemctl status nix-daemon".to_string()),
        )
        .await,
    );

    rows.push(config_syntax_check(&machine).await);
    rows.push(store_usage_check(&machine).await);
    rows.push(generation_count_check(&machine, keep_generations).await);

    if !machine.is_local() {
        rows.push(DiagnosticRow {
            label: "ssh connectivity",
            ok: true,
            detail: "ssh command path is reachable".to_string(),
            suggestion: None,
        });
    }

    Ok(DoctorReport {
        machine: machine.name,
        rows,
    })
}

async fn run_check(
    machine: &Machine,
    label: &'static str,
    command: &str,
    failure_detail: &str,
    suggestion: Option<String>,
) -> DiagnosticRow {
    match crate::exec::run(machine, command).await {
        Ok(output) if output.success() => DiagnosticRow {
            label,
            ok: true,
            detail: output
                .stdout
                .lines()
                .next()
                .unwrap_or("ok")
                .trim()
                .to_string(),
            suggestion: None,
        },
        Ok(output) => DiagnosticRow {
            label,
            ok: false,
            detail: if output.stderr.trim().is_empty() {
                failure_detail.to_string()
            } else {
                output.stderr.trim().to_string()
            },
            suggestion,
        },
        Err(err) => DiagnosticRow {
            label,
            ok: false,
            detail: err.to_string(),
            suggestion,
        },
    }
}

async fn config_syntax_check(machine: &Machine) -> DiagnosticRow {
    let command = format!(
        "nix-instantiate --parse {}/configuration.nix >/dev/null",
        machine.config_dir
    );
    run_check(
        machine,
        "config syntax",
        &command,
        "configuration.nix did not parse cleanly",
        Some("run: nina edit, then nina check".to_string()),
    )
    .await
}

async fn store_usage_check(machine: &Machine) -> DiagnosticRow {
    let command = "df -P /nix/store | awk 'NR==2 {print $5}'";
    match crate::exec::run(machine, command).await {
        Ok(output) if output.success() => {
            let detail = output.stdout.trim().to_string();
            let pct = detail.trim_end_matches('%').parse::<u32>().unwrap_or(0);
            DiagnosticRow {
                label: "disk space",
                ok: pct < 80,
                detail: format!("/nix/store is using {detail}"),
                suggestion: (pct >= 80).then(|| "try: nina clean".to_string()),
            }
        }
        Ok(output) => DiagnosticRow {
            label: "disk space",
            ok: false,
            detail: output.stderr.trim().to_string(),
            suggestion: None,
        },
        Err(err) => DiagnosticRow {
            label: "disk space",
            ok: false,
            detail: err.to_string(),
            suggestion: None,
        },
    }
}

async fn generation_count_check(machine: &Machine, keep_generations: u32) -> DiagnosticRow {
    let command = "nix-env --list-generations -p /nix/var/nix/profiles/system | wc -l";
    match crate::exec::run(machine, command).await {
        Ok(output) if output.success() => {
            let count = output.stdout.trim().parse::<u32>().unwrap_or(0);
            DiagnosticRow {
                label: "generation count",
                ok: count <= keep_generations,
                detail: format!("{count} generations found"),
                suggestion: (count > keep_generations).then(|| "try: nina clean".to_string()),
            }
        }
        Ok(output) => DiagnosticRow {
            label: "generation count",
            ok: false,
            detail: output.stderr.trim().to_string(),
            suggestion: None,
        },
        Err(err) => DiagnosticRow {
            label: "generation count",
            ok: false,
            detail: err.to_string(),
            suggestion: None,
        },
    }
}

fn render_report(ctx: &AppContext, report: &DoctorReport) {
    ctx.output
        .info(&format!("giving {} a little look-over...", report.machine));
    let mut warnings = 0;

    for row in &report.rows {
        if row.ok {
            ctx.output.kv_succ(row.label, &row.detail);
        } else {
            warnings += 1;
            ctx.output.kv_warn(row.label, &row.detail);
            if let Some(suggestion) = &row.suggestion {
                ctx.output.tip(suggestion);
            }
        }
    }

    if warnings == 0 {
        ctx.output
            .happy("nothing scary here, just keeping an eye on things ♡");
    } else {
        ctx.output.happy(&format!(
            "nothing broken, {} thing{} to tidy ♡",
            warnings,
            if warnings == 1 { "" } else { "s" }
        ));
    }
}
