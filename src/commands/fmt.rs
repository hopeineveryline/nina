use anyhow::{Context, Result};
use clap::Args;

use crate::commands::{confirm_action, AppContext};

#[derive(Debug, Clone, Args)]
pub struct FmtArgs {
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub check: bool,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: FmtArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let targets = gather_targets(&machine, args.all).await?;
    if targets.is_empty() {
        ctx.output.warn("i couldn't find any nix files to format.");
        return Ok(());
    }

    let mut changes = Vec::new();
    for path in targets {
        let original = if machine.is_local() {
            std::fs::read_to_string(&path).with_context(|| format!("couldn't read {path}"))?
        } else {
            crate::commands::edit::fetch_remote_file(&machine, &path).await?
        };
        let formatted = format_contents(&machine, &path, &original).await?;
        if formatted != original {
            changes.push((path, diff_preview(&original, &formatted), formatted));
        }
    }

    if args.check {
        if changes.is_empty() {
            ctx.output.success("everything is already formatted ♡");
            return Ok(());
        }
        ctx.output.warn("formatting changes are needed:");
        for (path, _, _) in &changes {
            ctx.output.print(&format!("  - {}", path));
        }
        anyhow::bail!("the formatting needs a little help — run nina fmt to fix it");
    }

    if changes.is_empty() {
        ctx.output.success("everything is already formatted ♡");
        return Ok(());
    }

    ctx.output.info("formatting nix files...");
    for (_, diff, _) in &changes {
        ctx.output.diff(diff);
    }

    if !confirm_action(ctx.config.confirm, "apply formatting?")? {
        ctx.output.warn("okay, cancelled with no changes ♡");
        return Ok(());
    }

    for (path, _, formatted) in changes {
        if machine.is_local() {
            std::fs::write(&path, formatted).with_context(|| format!("couldn't write {path}"))?;
        } else {
            crate::commands::edit::upload_remote_file(&machine, &path, &formatted).await?;
        }
    }
    ctx.output.success("formatted ♡");
    Ok(())
}

async fn gather_targets(machine: &crate::machine::Machine, all: bool) -> Result<Vec<String>> {
    if all {
        if machine.is_local() {
            let mut files = Vec::new();
            walk_nix_files(std::path::Path::new(&machine.config_dir), &mut files)?;
            return Ok(files);
        }
        let command = format!(
            "find {} -type f -name '*.nix'",
            crate::commands::shell_quote(&machine.config_dir)
        );
        let output = crate::exec::run(machine, &command).await?;
        if !output.success() {
            anyhow::bail!("couldn't list remote nix files: {}", output.stderr.trim());
        }
        return Ok(output.stdout.lines().map(ToString::to_string).collect());
    }
    Ok(vec![format!("{}/configuration.nix", machine.config_dir)])
}

fn walk_nix_files(dir: &std::path::Path, files: &mut Vec<String>) -> Result<()> {
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("couldn't read {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_nix_files(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("nix") {
            files.push(path.display().to_string());
        }
    }
    Ok(())
}

async fn format_contents(
    machine: &crate::machine::Machine,
    path: &str,
    original: &str,
) -> Result<String> {
    if machine.is_local() {
        let temp = std::env::temp_dir().join(format!("nina-fmt-{}.nix", std::process::id()));
        std::fs::write(&temp, original)
            .with_context(|| format!("couldn't write {}", temp.display()))?;
        let command = format!(
            "nixpkgs-fmt {}",
            crate::commands::shell_quote(&temp.display().to_string())
        );
        let output = crate::exec::run_local(&command, |_, _| {}).await?;
        if !output.success() {
            anyhow::bail!("formatting failed for {path}: {}", output.stderr.trim());
        }
        let formatted = std::fs::read_to_string(&temp)
            .with_context(|| format!("couldn't read {}", temp.display()))?;
        let _ = std::fs::remove_file(temp);
        return Ok(formatted);
    }

    let remote_temp = format!("/tmp/nina-fmt-{}.nix", std::process::id());
    crate::commands::edit::upload_remote_file(machine, &remote_temp, original).await?;
    let command = format!(
        "nixpkgs-fmt {} && cat {}",
        crate::commands::shell_quote(&remote_temp),
        crate::commands::shell_quote(&remote_temp)
    );
    let output = crate::exec::run(machine, &command).await?;
    if !output.success() {
        anyhow::bail!("formatting failed for {path}: {}", output.stderr.trim());
    }
    Ok(output.stdout)
}

fn diff_preview(before: &str, after: &str) -> String {
    let before_lines = before.lines().collect::<Vec<_>>();
    let after_lines = after.lines().collect::<Vec<_>>();
    let max_len = before_lines.len().max(after_lines.len());
    let mut out = vec!["changes:".to_string(), String::new()];

    for idx in 0..max_len {
        match (before_lines.get(idx), after_lines.get(idx)) {
            (Some(a), Some(b)) if a == b => {}
            (Some(a), Some(b)) => {
                out.push(format!("line {}:", idx + 1));
                out.push(format!("- {}", a));
                out.push(format!("+ {}", b));
                out.push(String::new());
            }
            (Some(a), None) => {
                out.push(format!("line {}:", idx + 1));
                out.push(format!("- {}", a));
                out.push(String::new());
            }
            (None, Some(b)) => {
                out.push(format!("line {}:", idx + 1));
                out.push(format!("+ {}", b));
                out.push(String::new());
            }
            (None, None) => {}
        }
    }
    out.join("\n").trim_end().to_string()
}
