use anyhow::Result;
use clap::Args;

use crate::commands::{current_dir_command, run_machine_command, AppContext};

#[derive(Debug, Clone, Args)]
pub struct UpdateArgs {
    pub input: Option<String>,
}

pub async fn run(ctx: &AppContext, args: UpdateArgs) -> Result<()> {
    let command = if let Some(input) = args.input {
        current_dir_command(&format!(
            "nix flake update --update-input {}",
            crate::commands::shell_quote(&input)
        ))?
    } else {
        current_dir_command("nix flake update")?
    };
    let on = None;
    run_machine_command(
        ctx,
        &on,
        "updating flake inputs",
        &command,
        "flake-update",
        false,
    )
    .await
}
