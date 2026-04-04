use anyhow::Result;
use clap::Args;

use crate::commands::{current_dir_command, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct BuildArgs {
    pub target: Option<String>,
}

pub async fn run(ctx: &AppContext, args: BuildArgs) -> Result<()> {
    let target = args.target.unwrap_or_default();
    let nix_cmd = if target.is_empty() {
        "nix build".to_string()
    } else if looks_like_url(&target) {
        format!("nix build {}", crate::commands::shell_quote(&target))
    } else {
        format!(
            "nix build {}",
            crate::commands::shell_quote(&format!(".#{}", target))
        )
    };
    let command = current_dir_command(&nix_cmd)?;
    let on = None;
    run_machine_command(ctx, &on, "building flake output", &command, "build", false).await
}

fn looks_like_url(target: &str) -> bool {
    target.contains("://") || target.starts_with("github:") || target.starts_with("git+")
}
