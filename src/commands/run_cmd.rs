use anyhow::Result;
use clap::Args;

use crate::commands::{current_dir_command, run_attached_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct RunArgs {
    pub target: Option<String>,
}

pub async fn run(ctx: &AppContext, args: RunArgs) -> Result<()> {
    let target = args.target.unwrap_or_default();
    let nix_cmd = if target.is_empty() {
        "nix run".to_string()
    } else if looks_like_url(&target) {
        format!(
            "nix run {} --no-write-lock-file",
            crate::commands::shell_quote(&target)
        )
    } else {
        format!(
            "nix run {}",
            crate::commands::shell_quote(&format!("nixpkgs#{}", target))
        )
    };
    let command = current_dir_command(&nix_cmd)?;
    let on = None;
    run_attached_machine_command(ctx, &on, "running flake app", &command, "run", false).await
}

fn looks_like_url(target: &str) -> bool {
    target.contains("://") || target.starts_with("github:") || target.starts_with("git+")
}
