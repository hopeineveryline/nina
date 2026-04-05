use anyhow::Result;
use clap::Args;
use serde::Deserialize;

use crate::commands::{confirm_action, AppContext};

#[derive(Debug, Clone, Args)]
pub struct UpdateArgs {
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: UpdateArgs) -> Result<()> {
    // Always run channel update on the target machine
    let machine = ctx.machine(&args.on)?;
    ctx.output
        .info(&format!("freshening channels on {}...", machine.name));
    ctx.output
        .step("sudo nix-channel --update");

    let result = crate::exec::run(&machine, "sudo nix-channel --update").await?;
    if !result.success() {
        ctx.output
            .error(&format!("channel update stumbled: {}", result.stderr.trim()));
        return Ok(());
    }
    ctx.output.success("channels are fresh ♡");

    // Check for nina updates locally (not on remote machine)
    if !args.on.is_some() {
        check_nina_update(ctx).await;
    }

    Ok(())
}

async fn check_nina_update(ctx: &AppContext) {
    let current = env!("CARGO_PKG_VERSION");

    let Ok(latest) = fetch_latest_release().await else {
        return; // Network issue or API failure — nothing to do
    };

    let Some(new_version) = parse_version(&latest.tag_name) else {
        return; // Couldn't parse the tag — skip
    };

    let current_ver = match parse_version(current) {
        Some(v) => v,
        None => return,
    };

    if new_version <= current_ver {
        return; // Already up to date
    }

    ctx.output.blank();
    ctx.output.info(&format!(
        "a newer nina is available: {} (you have {})",
        latest.tag_name.trim_start_matches('v'), current
    ));

    if let Some(body) = latest.body.as_ref() {
        let trimmed = body.trim();
        if !trimmed.is_empty() {
            ctx.output.print(&format!("  what's new:\n{}", indent(trimmed, "    ")));
        }
    }

    ctx.output.blank();
    let should_update = confirm_action(ctx.config.confirm, "update nina now?").unwrap_or(false);
    if should_update {
        ctx.output.blank();
        ctx.output.step("pulling the freshest nina...");

        let update_result = tokio::process::Command::new("sh")
            .args(["-lc", "nix run github:hopeineveryline/nina -- --version"])
            .output()
            .await;

        match update_result {
            Ok(out) if out.status.success() => {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !version.is_empty() {
                    ctx.output
                        .happy(&format!("nina updated to {} ♡", version.trim_start_matches("nina ")));
                } else {
                    ctx.output.happy("nina has been updated ♡");
                }
            }
            _ => {
                ctx.output
                    .warn("the nix run way didn't work — try:");
                ctx.output.print("  nix profile install github:hopeineveryline/nina");
                ctx.output.print("  nix run github:hopeineveryline/nina");
            }
        }
    } else {
        ctx.output
            .tip("you can update later with: nix run github:hopeineveryline/nina");
    }
}

async fn fetch_latest_release() -> Result<LatestRelease> {
    let client = reqwest::Client::builder()
        .user_agent("nina-cli/1.0")
        .build()?;
    let res = client
        .get("https://api.github.com/repos/hopeineveryline/nina/releases/latest")
        .send()
        .await?;
    if !res.status().is_success() {
        anyhow::bail!("GitHub API returned {}", res.status());
    }
    let release: LatestRelease = res.json().await?;
    Ok(release)
}

#[derive(Debug, Deserialize)]
struct LatestRelease {
    tag_name: String,
    body: Option<String>,
}

fn parse_version(tag: &str) -> Option<(u32, u32, u32)> {
    let version = tag.trim_start_matches('v');
    let mut parts = version.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    let patch: u32 = parts.next()?.split('-').next()?.parse().ok()?;
    Some((major, minor, patch))
}

fn indent(text: &str, prefix: &str) -> String {
    text.lines()
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
}
