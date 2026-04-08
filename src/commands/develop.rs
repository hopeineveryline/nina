use anyhow::{Context, Result};
use clap::Args;

use crate::commands::{
    current_dir_command, current_dir_command_for, run_attached_machine_command,
    run_machine_command, AppContext,
};

#[derive(Debug, Clone, Args)]
pub struct DevelopArgs {
    #[arg(long, num_args = 1.., trailing_var_arg = true)]
    pub run: Vec<String>,
    #[arg(long)]
    pub show: bool,
    #[arg(long)]
    pub on: Option<String>,
}

pub async fn run(ctx: &AppContext, args: DevelopArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    if !args.run.is_empty() {
        let cmd = args.run.join(" ");
        let command = current_dir_command_for(&machine, &format!("nix develop --command {}", cmd))?;
        return run_machine_command(
            ctx,
            &args.on,
            "running something in the dev shell",
            &command,
            "develop",
            false,
        )
        .await;
    }

    if args.show {
        return show_dev_shell(ctx, args).await;
    }

    if let Ok(packages) = read_dev_shell_packages(&machine).await {
        if !packages.is_empty() {
            ctx.output.section("packages in this shell");
            ctx.output.print(&format!("    {}", packages.join("  ")));
            ctx.output.blank();
        }
    }

    ctx.output.tip("type 'exit' to return to your normal shell");
    ctx.output.blank();
    let command = current_dir_command_for(&machine, "nix develop")?;
    run_attached_machine_command(
        ctx,
        &args.on,
        "slipping into the dev shell",
        &command,
        "develop",
        false,
    )
    .await?;
    ctx.output.success("back in your normal shell ♡");
    Ok(())
}

async fn show_dev_shell(ctx: &AppContext, args: DevelopArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    ctx.output.info("checking the default dev shell...");
    if let Ok(packages) = read_dev_shell_packages(&machine).await {
        if !packages.is_empty() {
            ctx.output.section("packages in this shell");
            ctx.output.print(&format!("    {}", packages.join("  ")));
            ctx.output.blank();
        }
    }

    let command = current_dir_command("nix flake show")?;
    run_machine_command(
        ctx,
        &args.on,
        "showing flake outputs",
        &command,
        "develop-show",
        false,
    )
    .await
}

async fn read_dev_shell_packages(machine: &crate::machine::Machine) -> Result<Vec<String>> {
    let expression = r#"let
  flake = builtins.getFlake (toString ./.);
  system = builtins.currentSystem;
  shell =
    if flake ? devShells && flake.devShells ? ${system} && flake.devShells.${system} ? default then flake.devShells.${system}.default
    else if flake ? devShell && flake.devShell ? ${system} then flake.devShell.${system}
    else throw \"no default dev shell\";
  inputs = (shell.nativeBuildInputs or []) ++ (shell.buildInputs or []);
  nameOf = pkg:
    let raw = pkg.pname or pkg.name or \"unknown\";
        parsed = if builtins.isString raw then builtins.parseDrvName raw else { name = \"unknown\"; };
    in parsed.name;
    in builtins.map nameOf inputs"#;
    let command = current_dir_command_for(
        machine,
        &format!(
            "nix eval --json --impure --expr {}",
            crate::commands::shell_quote(expression)
        ),
    )?;
    let output = crate::exec::run(machine, &command).await?;
    if !output.success() {
        return Ok(Vec::new());
    }
    serde_json::from_str::<Vec<String>>(&output.stdout).context("couldn't parse dev shell packages")
}
